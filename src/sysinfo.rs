use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

/// Previous CPU sample for delta-based calculation
#[derive(Debug, Clone)]
struct CpuSample {
    total_ticks: u64,  // utime + stime
    timestamp: Instant,
    uptime: f64,       // system uptime at sample time
}

/// Global store of previous CPU samples keyed by PID
static PREV_SAMPLES: Mutex<Option<HashMap<u32, CpuSample>>> = Mutex::new(None);

#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub cpu_percent: f64,
    pub mem_mb: u64,
    pub mem_percent: f64,
}

/// Get CPU and memory stats for all processes in a tmux session
pub fn get_session_stats(session_name: &str) -> Result<SessionStats> {
    let pids = get_session_pids(session_name)?;

    if pids.is_empty() {
        return Ok(SessionStats::default());
    }

    let now = Instant::now();
    let uptime = get_system_uptime()?;

    let mut prev_map = PREV_SAMPLES.lock().unwrap();
    let prev = prev_map.get_or_insert_with(HashMap::new);

    let mut total_cpu = 0.0;
    let mut total_mem_kb = 0u64;
    let _num_cpus = get_num_cpus();

    for pid in &pids {
        if let Ok((ticks, mem)) = get_process_raw(*pid) {
            total_mem_kb += mem;

            // Delta-based CPU: compare with previous sample
            if let Some(old) = prev.get(pid) {
                let dt = uptime - old.uptime;
                if dt > 0.01 {
                    let dticks = ticks.saturating_sub(old.total_ticks);
                    let hertz = 100.0; // USER_HZ
                    let cpu = (dticks as f64 / hertz) / dt * 100.0;
                    total_cpu += cpu;
                }
            }
            // else: first sample for this PID, CPU will be 0 this round

            prev.insert(*pid, CpuSample {
                total_ticks: ticks,
                timestamp: now,
                uptime,
            });
        }
    }

    // Clean stale PIDs (not seen in 30 seconds)
    prev.retain(|_, sample| now.duration_since(sample.timestamp).as_secs() < 30);

    let mem_mb = total_mem_kb / 1024;
    let total_mem_kb_sys = get_total_memory_kb().unwrap_or(1);
    let mem_percent = (total_mem_kb as f64 / total_mem_kb_sys as f64) * 100.0;

    Ok(SessionStats {
        cpu_percent: total_cpu,
        mem_mb,
        mem_percent,
    })
}

/// Get all PIDs for processes in a tmux session
fn get_session_pids(session_name: &str) -> Result<Vec<u32>> {
    let output = Command::new("tmux")
        .args(["list-panes", "-t", session_name, "-a", "-F", "#{pane_pid}"])
        .output()
        .context("Failed to get pane PIDs")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut pids = Vec::new();

    for line in stdout.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            pids.push(pid);
            if let Ok(descendants) = get_descendant_pids(pid) {
                pids.extend(descendants);
            }
        }
    }

    Ok(pids)
}

/// Get all descendant PIDs of a given PID
fn get_descendant_pids(pid: u32) -> Result<Vec<u32>> {
    let output = Command::new("pgrep")
        .args(["-P", &pid.to_string()])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut descendants = Vec::new();

    for line in stdout.lines() {
        if let Ok(child_pid) = line.trim().parse::<u32>() {
            descendants.push(child_pid);
            if let Ok(grand_children) = get_descendant_pids(child_pid) {
                descendants.extend(grand_children);
            }
        }
    }

    Ok(descendants)
}

/// Get raw CPU ticks (utime+stime) and memory (KB) for a process
fn get_process_raw(pid: u32) -> Result<(u64, u64)> {
    let stat_path = format!("/proc/{}/stat", pid);
    let statm_path = format!("/proc/{}/statm", pid);

    let stat_content = fs::read_to_string(&stat_path).context("Failed to read stat")?;
    let ticks = parse_ticks_from_stat(&stat_content)?;

    let statm_content = fs::read_to_string(&statm_path).context("Failed to read statm")?;
    let mem_kb = parse_mem_from_statm(&statm_content)?;

    Ok((ticks, mem_kb))
}

fn parse_ticks_from_stat(content: &str) -> Result<u64> {
    let rest = content
        .split(") ")
        .nth(1)
        .context("Invalid stat format")?;

    let fields: Vec<&str> = rest.split_whitespace().collect();

    if fields.len() < 13 {
        return Ok(0);
    }

    let utime: u64 = fields[11].parse().unwrap_or(0);
    let stime: u64 = fields[12].parse().unwrap_or(0);

    Ok(utime + stime)
}

fn parse_mem_from_statm(content: &str) -> Result<u64> {
    let fields: Vec<&str> = content.split_whitespace().collect();
    if fields.is_empty() {
        return Ok(0);
    }
    // Use RSS (field 1) instead of total VM size (field 0) for actual memory usage
    let pages: u64 = if fields.len() > 1 {
        fields[1].parse().unwrap_or(0)
    } else {
        fields[0].parse().unwrap_or(0)
    };
    let page_size_kb = 4;
    Ok(pages * page_size_kb)
}

fn get_system_uptime() -> Result<f64> {
    let content = fs::read_to_string("/proc/uptime").context("Failed to read uptime")?;
    let uptime_str = content.split_whitespace().next().context("Invalid uptime")?;
    uptime_str.parse().context("Failed to parse uptime")
}

fn get_total_memory_kb() -> Result<u64> {
    let content = fs::read_to_string("/proc/meminfo").context("Failed to read meminfo")?;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().context("Failed to parse MemTotal");
            }
        }
    }

    Ok(8 * 1024 * 1024) // Default to 8GB
}

fn get_num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_total_memory() {
        let mem = get_total_memory_kb();
        assert!(mem.is_ok());
        assert!(mem.unwrap() > 0);
    }

    #[test]
    fn test_get_system_uptime() {
        let uptime = get_system_uptime();
        assert!(uptime.is_ok());
        assert!(uptime.unwrap() > 0.0);
    }

    #[test]
    fn test_delta_cpu_needs_two_samples() {
        // Should be able to read raw stats for current process
        let pid = std::process::id();
        let result = get_process_raw(pid);
        assert!(result.is_ok());
    }
}
