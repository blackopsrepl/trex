# Project: Group Child AI Processes Under Parent

Display AI processes that are children of other AI processes (e.g., claude spawned by zoyd) as grouped entries rather than separate entries.

**Before:** `⏸ zoyd:myproject ○` and `⏸ claude:myproject ○` shown separately  
**After:** `⏸ zoyd:myproject ○ (claude)` as a single entry

## Tasks

### Process Detection Changes
- [x] Add `child_ai_names: Vec<String>` field to `AiProcessInfo` struct (src/process.rs)
- [x] Initialize `child_ai_names: Vec::new()` in `get_process_info()` (src/process.rs)
- [x] Modify `find_ai_processes()` to detect parent-child relationships among AI processes and populate `child_ai_names` (src/process.rs)
- [x] Filter out child AI processes from the returned list, keeping only root AI processes (src/process.rs)

### Display Changes
- [x] Increase `COL_WIDTH` constant from 30 to 38 (src/tui/ui/normal.rs)
- [x] Update agent display format to append child names after tmux icon, e.g., ` (claude, opencode)` (src/tui/ui/normal.rs)

### Testing
- [x] Build and verify no compilation errors
- [x] Manual test: run trex and verify grouped display works correctly

## Notes

- Use existing `get_ppid()` function to get parent PID
- First pass collects all AI processes, second pass links parent-child relationships
- Only immediate parent-child relationships need detection (no deep nesting)
- Multiple children should be comma-separated: `(claude, opencode)`
- Child process names use short form only (no project name suffix)
- Standalone AI processes (not children) display unchanged

## Success Criteria

- All checkboxes marked `[x]`
- When zoyd spawns claude, only one entry appears: `zoyd:project ○ (claude)`
- When claude runs standalone, it appears normally: `claude:project ●`
- Multiple children display correctly: `zoyd:project ○ (claude, opencode)`
