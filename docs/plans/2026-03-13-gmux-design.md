# gmux — Design Document

A lightweight terminal management app for AI coding agents.
Inspired by cmux (macOS-only) but cross-platform, starting with Linux.

## Tech Stack

| Component | Version | Technology |
|-----------|---------|------------|
| Framework | 2.10.3 | Tauri v2 (Rust Backend) |
| Frontend | 5.53.9 | Svelte 5 (Runes API) + TypeScript |
| Terminal | 6.0.0 | @xterm/xterm + WebGL Addon |
| PTY | 0.9.0 | Rust portable-pty |
| Bundler | 8.0.0 | Vite (Rolldown) |
| Styling | - | CSS Variables (Dark Theme) |
| State | - | Svelte Stores ($state, $derived) |
| Git | 0.20.4 | Rust git2 |
| Notifications | - | Tauri Notification Plugin |
| Storage | - | JSON Config Files (~/.config/gmux/) |

## Design Palette

| Property | Value |
|----------|-------|
| Background | #171717 |
| Sidebar | #0d0d0d |
| Surface/Cards | #2a2a2a |
| Borders | #303030 |
| Text Primary | #e5e5e5 |
| Text Secondary | #8e8ea0 |
| Accent | #10a37f |
| Diff Add | #212922 |
| Diff Delete | #3c170f |
| Notification | #3b82f6 |
| Border Radius | 8px containers, 6px buttons |
| Font UI | Inter / System Sans |
| Font Code | JetBrains Mono / System Mono |

## Layout — Terminal View (Default)

```
+-----------------------------------------------------------------------------+
| * * *                        gmux                 [Terminals] [Insights] [S] |
+---------------------+-------------------------------------------------------+
|                     |                                                       |
|  [+ New Workspace]  |  +------------------------+------------------------+  |
|                     |  | claude-1 bypass        | claude-2 bypass        |  |
|  WORKSPACES         |  |                        |                        |  |
|                     |  |  $ claude              |  $ claude              |  |
|  > Frontend Work    |  |  > Analyzing your      |  > Running tests...    |  |
|    * claude-1       |  |    codebase...         |                        |  |
|      claude-2       |  |                        |                        |  |
|      npm-dev        |  |                        |                        |  |
|                     |  |                        |                        |  |
|  > API Backend      |  +------------------------+------------------------+  |
|      codex-1        |  | npm-dev                | jest-watch             |  |
|      jest-watch     |  |                        |                        |  |
|                     |  |  $ npm run dev         |  $ jest --watch        |  |
|  > Quick Terminal   |  |  Server on :3000       |  42 tests passed       |  |
|      shell          |  |                        |                        |  |
|                     |  |                        |                        |  |
|  ----------------   |  +------------------------+------------------------+  |
|  [New Folder]       |                                                       |
|                     |  [New Workspace]                       [+ Terminal]   |
|              [<]    |                                                       |
+---------------------+-------------------------------------------------------+
| main * claude-opus-4 * reasoning: high * u12k d8k * ctx 32% (64k/200k)     |
+-----------------------------------------------------------------------------+
```

Terminal area takes ~90% of space. Clean, no bloat.

### Sidebar (~250px default)

- Resizable via drag on edge
- Minimizable via [<] button to ~48px icon-only (badges remain visible)
- [+ New Workspace] opens creation modal
- Workspaces grouped in Folders
- Sessions as children of workspaces
- Notification badge (blue dot + number) on sessions needing input
- Status text per session (Running, Needs input, Agent is ready)
- Right-click context menu: Rename, Move, Duplicate, Close
- Drag & drop sessions between folders

### Terminal Area

- Splits: vertical AND horizontal, freely combinable
- Resize via drag on split borders
- Pane header: Name + Bypass Badge + [Split] + [Close]
- Tabs within panes for multiple terminals
- [+ Terminal] adds single terminal to active workspace

### Statusbar

`main * claude-opus-4 * reasoning: high * u12k d8k * ctx 32% (64k/200k)`

- Git branch from active terminal's working directory
- Model name parsed from terminal output
- Reasoning mode parsed from terminal output
- Token count (input/output)
- Context with absolute numbers

## Insights View

Click [Insights] in top bar replaces terminal area. Sidebar becomes insights navigation.

### Usage Tab

- Token breakdown: Input, Output, Cache Read, Cache Write + costs
- Today / Weekly / Monthly toggle
- Rate limits with progress bars + reset countdown
  - Per provider configurable (5h window, weekly limit, reset times)
- Per-session breakdown
- Manual refresh button (no auto-refresh)
- Data from real ccusage / Claude Code JSONL session files

### Git Tab

- Branch dropdown to switch branches
- Uncommitted changes list with +/- line counts
- Click on file opens inline diff view
  - Red background = deleted lines
  - Green background = added lines
  - Line numbers on both sides
- Stage/Unstage/Revert per file
- Real data via Rust git2 crate
- Manual refresh button

### Info Tab

- App version, system info
- Configured AI CLIs + paths
- Active sessions overview

## New Workspace Modal

Single-screen modal (no multi-step wizard):

- **Name:** text input for workspace name
- **Path:** directory input with browse button + Recent dropdown
  - Recent paths stored zoxide-style (frequency + recency = score)
  - Clickable dropdown, most relevant paths first
- **Layout Templates:** visual grid selection
  - 1 terminal, 2 side-by-side, 2 vertical, 4 grid, 3 columns, 6 grid
- **Agents (optional):**
  - Claude Code with +/- counter + bypass permissions checkbox
  - Codex CLI with +/- counter
  - Gemini CLI with +/- counter
  - Shell with +/- counter
  - Slot counter showing assigned/total
  - All at 0 = plain terminal workspace
  - Unassigned slots auto-fill with empty shells

### Claude Bypass Permissions

- Checkbox per Claude agent: "bypass permissions"
- When active: starts with `claude --dangerously-skip-permissions`
- Visible as badge in terminal pane header

### Instant Terminal Launch

- Rust backend spawns ALL PTYs in parallel (tokio::spawn / thread::spawn)
- Each PTY receives its command immediately:
  - Claude: `claude` or `claude --dangerously-skip-permissions`
  - Codex: `codex --dangerously-bypass-approvals-and-sandbox`
  - Gemini: `gemini`
  - Shell: user's default shell
- Commands auto-sent via PTY write + Enter
- All terminals open in < 2 seconds regardless of count

## Terminal Features

- @xterm/xterm 6.0.0 + WebGL Addon for GPU-accelerated rendering
- Shift+Enter for newline (no submit)
- Ctrl+V for text paste / image paste (depending on clipboard content)
- Full ANSI/256-Color/TrueColor support
- Configurable scrollback buffer
- Search via Ctrl+Shift+F
- Splits via right-click menu or keyboard shortcuts
- Resize via drag on borders
- Tabs within panes for multiple terminals

## Notification System

- Pattern detection in terminal output via configurable regex
- Default patterns:
  - Claude: prompt markers, `? (y/n)`, permission prompts, `Waiting for input`
  - Codex: `? for shortcuts`, idle states
  - Gemini: input prompts
- Blue badge + count in sidebar
- Desktop notification via Tauri native notification API
- Custom patterns configurable in settings

## Session Persistence

Everything persists on app restart via JSON files in ~/.config/gmux/:

**Persisted:**
- Workspaces, folders, terminal layouts, split positions
- Sidebar width, minimized state
- Working directories, workspace names
- Terminal scrollback buffer content
- Recent paths (zoxide-style scores)
- All settings, keybindings
- Window size and position

**Not persisted (technically impossible):**
- Running processes (Claude, npm, etc.) must be restarted
- Layout + scrollback remain so previous output is visible

## Settings

- **Appearance:** Theme, Accent Color, Font UI, Font Code, Font Size
- **Terminal:** Default Shell, Scrollback Lines, Cursor Style (block/beam/underline)
- **AI CLIs:** Paths to claude, codex, gemini + custom commands
- **Rate Limits:** Per provider configurable (5h window size, weekly limit, reset times)
- **Notifications:** Desktop notifications on/off, sound, custom detection patterns (regex)
- **Keybindings:** All shortcuts customizable

## Code Rules

ABSOLUTE rules with ZERO exceptions:

1. NEVER use mock data, placeholders, dummy values, or sample data anywhere
2. NEVER write code comments of any kind — no inline comments, no block comments, no JSX comments, no TODO comments, no descriptive comments
3. NEVER write `{/* ... */}` JSX/Svelte comments
4. NEVER write `// ...` or `/* ... */` placeholder comments
5. NEVER write `// TODO`, `// FIXME`, `// HACK`, `// NOTE` or similar
6. NEVER hardcode values — use environment variables or config files
7. NEVER use placeholder text like "Lorem ipsum" or "Coming soon"
8. NEVER create empty functions or stub implementations
9. NEVER leave unused imports, variables, or dead code
10. Every function must have a real implementation that does real work
11. Every API call must be to a real endpoint with real data
12. Every UI element must display real, live data from the actual system
13. Clean Code means: self-documenting function/variable names, small focused functions, single responsibility
14. If something is not implemented yet, do not render it — omit it entirely
