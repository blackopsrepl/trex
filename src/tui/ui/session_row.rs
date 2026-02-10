use crate::tmux::ActivityLevel;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Gauge, Paragraph, Sparkline},
};

pub fn render_session_header(
    frame: &mut Frame,
    app: &App,
    inner: Rect,
    y_offset: &mut u16,
    session: &crate::tmux::TmuxSession,
    is_selected: bool,
) {
    let (activity_icon, activity_color) = match session.activity_level() {
        Some(ActivityLevel::Active) => ("●", app.theme.success),
        Some(ActivityLevel::Idle) => ("○", app.theme.warning),
        Some(ActivityLevel::Dormant) => ("◌", app.theme.text_dim),
        None => ("○", app.theme.text_dim),
    };

    let attached_indicator = if session.attached { "★" } else { "☆" };
    let name_style = if is_selected {
        Style::default()
            .fg(app.theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.text)
    };

    let activity_ago = session.activity_ago_string().unwrap_or_default();
    let git_badge = session
        .git_status
        .as_ref()
        .and_then(|gs| gs.badge())
        .unwrap_or_default();

    // Calculate health score
    let health = crate::health::HealthScore::calculate(session);
    let health_icon = health.icon();
    let health_color = match health.level() {
        crate::health::HealthLevel::Healthy => app.theme.success,
        crate::health::HealthLevel::Warning => app.theme.warning,
        crate::health::HealthLevel::Critical => app.theme.error,
    };

    let header_line = Line::from(vec![
        Span::styled(activity_icon, Style::default().fg(activity_color)),
        Span::raw(" "),
        Span::styled(attached_indicator, Style::default().fg(app.theme.warning)),
        Span::raw(" "),
        Span::styled(&session.name, name_style),
        Span::raw(" "),
        Span::styled(health_icon, Style::default().fg(health_color)),
        Span::styled(
            format!(" ({} win)", session.windows),
            Style::default().fg(app.theme.text_dim),
        ),
        Span::styled(
            format!(" {}", activity_ago),
            Style::default().fg(activity_color),
        ),
        if !git_badge.is_empty() {
            Span::styled(
                format!(" {}", git_badge),
                Style::default().fg(app.theme.secondary),
            )
        } else {
            Span::raw("")
        },
    ]);

    let header_area = Rect {
        x: inner.x,
        y: inner.y + *y_offset,
        width: inner.width,
        height: 1,
    };

    let bg_style = if is_selected {
        Style::default().bg(app.theme.bg_highlight)
    } else {
        Style::default()
    };

    let header_para = Paragraph::new(header_line).style(bg_style);
    frame.render_widget(header_para, header_area);
    *y_offset += 1;
}

pub fn render_session_gauges(
    frame: &mut Frame,
    app: &App,
    inner: Rect,
    y_offset: &mut u16,
    session: &crate::tmux::TmuxSession,
) {
    if let Some(ref stats) = session.stats {
        // CPU Gauge with smooth gradient color
        let cpu_ratio = (stats.cpu_percent / 100.0).min(1.0);
        let cpu_color = app.theme.gradient_color(stats.cpu_percent);

        let cpu_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(cpu_color).bg(app.theme.bg_primary))
            .label(format!("CPU {:5.1}%", stats.cpu_percent))
            .ratio(cpu_ratio);

        let cpu_area = Rect {
            x: inner.x,
            y: inner.y + *y_offset,
            width: inner.width / 2,
            height: 1,
        };
        frame.render_widget(cpu_gauge, cpu_area);

        // Memory Gauge with smooth gradient color
        let mem_ratio = (stats.mem_percent / 100.0).min(1.0);
        let mem_color = app.theme.gradient_color(stats.mem_percent);

        let mem_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(mem_color).bg(app.theme.bg_primary))
            .label(format!("MEM {:4}MB", stats.mem_mb))
            .ratio(mem_ratio);

        let mem_area = Rect {
            x: inner.x + inner.width / 2,
            y: inner.y + *y_offset,
            width: inner.width / 2,
            height: 1,
        };
        frame.render_widget(mem_gauge, mem_area);
        *y_offset += 1;

        // CPU Sparkline
        if !session.cpu_history.is_empty() {
            let cpu_sparkline = Sparkline::default()
                .block(Block::default())
                .data(&session.cpu_history)
                .style(Style::default().fg(cpu_color).bg(app.theme.bg_primary));

            let cpu_spark_area = Rect {
                x: inner.x,
                y: inner.y + *y_offset,
                width: inner.width / 2,
                height: 1,
            };
            frame.render_widget(cpu_sparkline, cpu_spark_area);
        }

        // Memory Sparkline
        if !session.mem_history.is_empty() {
            let mem_sparkline = Sparkline::default()
                .block(Block::default())
                .data(&session.mem_history)
                .style(Style::default().fg(mem_color).bg(app.theme.bg_primary));

            let mem_spark_area = Rect {
                x: inner.x + inner.width / 2,
                y: inner.y + *y_offset,
                width: inner.width / 2,
                height: 1,
            };
            frame.render_widget(mem_sparkline, mem_spark_area);
        }
        *y_offset += 1;
    } else {
        // No stats available yet
        let waiting_line = Line::from(vec![Span::styled(
            "  ⏳ Gathering metrics...",
            Style::default().fg(app.theme.text_dim),
        )]);
        let waiting_area = Rect {
            x: inner.x,
            y: inner.y + *y_offset,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(waiting_line), waiting_area);
        *y_offset += 1;
    }
}
