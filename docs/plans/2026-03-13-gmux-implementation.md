# gmux Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a lightweight Tauri v2 terminal management app with workspace organization, AI agent launching, usage tracking, and git integration.

**Architecture:** Tauri v2 Rust backend manages PTY processes via portable-pty, communicates with the Svelte 5 frontend through Tauri's Channel API (PTY→frontend streaming) and invoke commands (frontend→PTY writes). State persisted as JSON in ~/.config/gmux/. xterm.js renders terminals with WebGL acceleration.

**Tech Stack:** Tauri 2.10.3, Svelte 5.53.9 (Runes), @xterm/xterm 6.0.0, portable-pty 0.9.0, git2 0.20.4, Vite 8.0.0

**Design Doc:** `docs/plans/2026-03-13-gmux-design.md`

**Code Rules (ZERO EXCEPTIONS):**
1. NEVER mock data, placeholders, dummy values, sample data
2. NEVER write code comments of any kind
3. NEVER hardcode values — use config
4. NEVER create empty functions or stubs
5. NEVER leave unused imports/variables/dead code
6. Every UI element displays real live data
7. If not implemented yet, omit entirely — do not render

---

## Phase 1: Project Scaffold

### Task 1: Initialize Tauri + Svelte 5 project

**Files:**
- Create: entire project structure

**Step 1: Scaffold the project**

Run:
```bash
cd /run/media/eriks/Volume/Projekt-Fertig/gmux
npm create tauri-app@latest . -- --template svelte-ts --manager npm
```

If the interactive scaffolder doesn't accept args, run interactively and select:
- Project name: gmux
- Frontend: TypeScript
- Package manager: npm
- UI template: Svelte
- UI flavor: TypeScript

**Step 2: Verify scaffold works**

Run:
```bash
cd /run/media/eriks/Volume/Projekt-Fertig/gmux
npm install
npm run tauri dev
```

Expected: Tauri window opens with default Svelte template.

**Step 3: Clean up scaffold boilerplate**

Remove all default Svelte demo content (counter, logos, etc.). Leave only a minimal App.svelte with an empty div.

**Step 4: Install frontend dependencies**

Run:
```bash
npm install @xterm/xterm @xterm/addon-fit @xterm/addon-webgl @xterm/addon-web-links @xterm/addon-search
```

**Step 5: Add Rust dependencies to src-tauri/Cargo.toml**

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-notification = "2"
tauri-plugin-dialog = "2"
portable-pty = "0.9"
git2 = "0.20"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
dirs = "6"
chrono = { version = "0.4", features = ["serde"] }
glob = "0.3"
regex = "1"
```

**Step 6: Set up project directory structure**

```
src/
  lib/
    components/
      sidebar/
      terminal/
      insights/
      workspace/
      settings/
      statusbar/
    stores/
    types/
    utils/
  App.svelte
  main.ts
  app.css
src-tauri/
  src/
    main.rs
    lib.rs
    pty/
      mod.rs
      manager.rs
    git/
      mod.rs
    config/
      mod.rs
    usage/
      mod.rs
```

Create empty directories and module files with `mod` declarations only.

**Step 7: Set up CSS variables for the design palette**

File: `src/app.css`

Define all CSS custom properties from the design palette:
- `--bg-primary: #171717`
- `--bg-sidebar: #0d0d0d`
- `--bg-surface: #2a2a2a`
- `--border-color: #303030`
- `--text-primary: #e5e5e5`
- `--text-secondary: #8e8ea0`
- `--accent: #10a37f`
- `--diff-add: #212922`
- `--diff-delete: #3c170f`
- `--notification: #3b82f6`
- `--radius-container: 8px`
- `--radius-button: 6px`
- Font families: Inter for UI, JetBrains Mono for code
- Reset: margin 0, padding 0, box-sizing border-box
- html/body: 100% height, overflow hidden, bg-primary, text-primary

**Step 8: Configure Tauri window**

File: `src-tauri/tauri.conf.json`

Set:
- title: "gmux"
- width: 1400, height: 900
- minWidth: 800, minHeight: 600
- decorations: true
- transparent: false
- resizable: true

**Step 9: Commit**

```bash
git init
git add -A
git commit -m "feat: scaffold Tauri + Svelte 5 project with dependencies"
```

---

## Phase 2: PTY Backend (Rust)

### Task 2: Build the PTY manager

**Files:**
- Create: `src-tauri/src/pty/mod.rs`
- Create: `src-tauri/src/pty/manager.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Implement PtyInstance struct**

File: `src-tauri/src/pty/manager.rs`

Build a `PtyInstance` struct that holds:
- `id: String` (uuid)
- `writer: Arc<Mutex<Box<dyn Write + Send>>>`
- `master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>`
- `child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>`
- `working_dir: String`
- `command: String`

Methods:
- `write(&self, data: &str) -> Result<()>` — write bytes to PTY
- `resize(&self, rows: u16, cols: u16) -> Result<()>` — resize PTY
- `kill(&self) -> Result<()>` — kill child process
- `is_alive(&self) -> bool` — check if process is running

**Step 2: Implement PtyManager**

File: `src-tauri/src/pty/manager.rs`

Build a `PtyManager` struct:
- `instances: Arc<Mutex<HashMap<String, PtyInstance>>>`

Methods:
- `spawn(shell: &str, cwd: &str, cols: u16, rows: u16, env_vars: HashMap<String, String>) -> Result<(String, Box<dyn Read + Send>)>` — creates PTY, spawns process, returns (id, reader). Sets TERM=xterm-256color. Uses native_pty_system().
- `write(id: &str, data: &str) -> Result<()>`
- `resize(id: &str, rows: u16, cols: u16) -> Result<()>`
- `kill(id: &str) -> Result<()>`
- `kill_all() -> Result<()>`
- `list() -> Vec<String>`

**Step 3: Expose module**

File: `src-tauri/src/pty/mod.rs`

Re-export PtyManager.

**Step 4: Run `cargo check` in src-tauri**

Run: `cd src-tauri && cargo check`
Expected: compiles without errors

**Step 5: Commit**

```bash
git add src-tauri/src/pty/
git commit -m "feat: implement PTY manager with spawn/write/resize/kill"
```

### Task 3: Tauri commands for PTY IPC

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/commands.rs`

**Step 1: Create Tauri commands**

File: `src-tauri/src/commands.rs`

Commands using Tauri v2 Channel API for streaming:

```rust
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
enum TerminalEvent {
    Output { data: Vec<u8> },
    Exit { code: Option<u32> },
}
```

- `create_pty(shell: String, cwd: String, cols: u16, rows: u16, on_event: Channel<TerminalEvent>) -> Result<String>` — spawns PTY, starts reader thread that streams output through channel, returns PTY id
- `write_pty(id: String, data: String) -> Result<()>`
- `resize_pty(id: String, rows: u16, cols: u16) -> Result<()>`
- `kill_pty(id: String) -> Result<()>`

The reader thread:
- Reads from PTY in a loop with 4096-byte buffer
- Sends each chunk as `TerminalEvent::Output` through the channel
- On EOF or error, sends `TerminalEvent::Exit`
- Uses `tauri::async_runtime::spawn_blocking`

**Step 2: Register commands and state in lib.rs**

File: `src-tauri/src/lib.rs`

- Create `PtyManager` as managed state: `.manage(Arc::new(Mutex::new(PtyManager::new())))`
- Register all PTY commands in `invoke_handler`
- Register notification plugin: `.plugin(tauri_plugin_notification::init())`
- Register dialog plugin: `.plugin(tauri_plugin_dialog::init())`

**Step 3: Run `cargo check`**

Run: `cd src-tauri && cargo check`
Expected: compiles

**Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add Tauri IPC commands for PTY with Channel streaming"
```

---

## Phase 3: Basic Terminal Component

### Task 4: Create the xterm.js terminal component

**Files:**
- Create: `src/lib/components/terminal/TerminalPane.svelte`
- Create: `src/lib/components/terminal/terminal-bridge.ts`

**Step 1: Build the Tauri-xterm bridge**

File: `src/lib/components/terminal/terminal-bridge.ts`

TypeScript module that:
- Imports `invoke` and `Channel` from `@tauri-apps/api/core`
- Defines `TerminalEvent` type matching Rust enum
- `createPty(shell, cwd, cols, rows, onData, onExit)` — creates Channel, sets up onmessage handler that calls onData with decoded output bytes, calls invoke('create_pty')
- `writePty(id, data)` — calls invoke('write_pty')
- `resizePty(id, rows, cols)` — calls invoke('resize_pty')
- `killPty(id)` — calls invoke('kill_pty')

Use `TextDecoder` to decode `Uint8Array` from the channel to string for xterm.

**Step 2: Build TerminalPane component**

File: `src/lib/components/terminal/TerminalPane.svelte`

Svelte 5 component with runes:
- Props: `terminalId: string`, `shell: string`, `cwd: string`, `command?: string`, `onTitleChange?: (title: string) => void`
- `$state` for: `ptyId`, `isAlive`
- `$effect` for: initialization (mount), cleanup (unmount)
- On mount:
  1. Create `Terminal` instance with theme from CSS variables
  2. Load `FitAddon`, `WebglAddon`, `WebLinksAddon`
  3. Call `term.open(containerEl)`
  4. Call `fitAddon.fit()`
  5. Call `createPty()` with shell, cwd, cols, rows
  6. Wire `term.onData()` → `writePty()`
  7. Wire channel output → `term.write()`
  8. If `command` prop is set, after 100ms delay write command + '\r' to PTY (for auto-launching claude etc.)
- `ResizeObserver` on container → `fitAddon.fit()` + `resizePty()`
- On unmount: dispose terminal, kill PTY

**Step 3: Test manually**

Run: `npm run tauri dev`
Temporarily render a single `TerminalPane` in App.svelte with shell set to user's default shell.
Expected: working terminal with input/output.

**Step 4: Commit**

```bash
git add src/lib/components/terminal/
git commit -m "feat: xterm.js terminal component with Tauri PTY bridge"
```

---

## Phase 4: App Shell & Layout

### Task 5: Build the app shell layout

**Files:**
- Create: `src/lib/stores/app.svelte.ts`
- Create: `src/lib/types/workspace.ts`
- Modify: `src/App.svelte`
- Create: `src/lib/components/TopBar.svelte`
- Create: `src/lib/components/statusbar/StatusBar.svelte`

**Step 1: Define types**

File: `src/lib/types/workspace.ts`

```typescript
interface TerminalSession {
  id: string;
  name: string;
  shell: string;
  cwd: string;
  command?: string;
  bypassPermissions?: boolean;
  status: 'running' | 'needs-input' | 'ready' | 'exited';
  notificationCount: number;
}

interface SplitNode {
  type: 'terminal' | 'split';
  direction?: 'horizontal' | 'vertical';
  ratio?: number;
  terminalId?: string;
  children?: SplitNode[];
}

interface Workspace {
  id: string;
  name: string;
  folderId?: string;
  cwd: string;
  layout: SplitNode;
  sessions: TerminalSession[];
  createdAt: string;
}

interface Folder {
  id: string;
  name: string;
  collapsed: boolean;
}

type AppView = 'terminals' | 'insights';
type InsightsTab = 'usage' | 'git' | 'info';
```

Export all types.

**Step 2: Create app store**

File: `src/lib/stores/app.svelte.ts`

Using Svelte 5 runes (class-based store pattern):

```typescript
class AppStore {
  workspaces = $state<Workspace[]>([]);
  folders = $state<Folder[]>([]);
  activeWorkspaceId = $state<string | null>(null);
  activeView = $state<AppView>('terminals');
  activeInsightsTab = $state<InsightsTab>('usage');
  sidebarWidth = $state(250);
  sidebarMinimized = $state(false);
  activeTerminalId = $state<string | null>(null);

  get activeWorkspace() { /* return workspace by activeWorkspaceId */ }
  get workspacesByFolder() { /* group workspaces by folderId */ }
  get ungroupedWorkspaces() { /* workspaces without folder */ }

  addWorkspace(ws: Workspace) { ... }
  removeWorkspace(id: string) { ... }
  setActiveWorkspace(id: string) { ... }
  addFolder(name: string) { ... }
  renameFolder(id: string, name: string) { ... }
  removeFolder(id: string) { ... }
  toggleSidebar() { ... }
  addSessionToWorkspace(workspaceId: string, session: TerminalSession) { ... }
  updateSessionStatus(sessionId: string, status: string) { ... }
  incrementNotification(sessionId: string) { ... }
  clearNotification(sessionId: string) { ... }
}

export const appStore = new AppStore();
```

**Step 3: Build TopBar**

File: `src/lib/components/TopBar.svelte`

- Renders: gmux title centered, [Terminals] [Insights] tab buttons on right, [Settings gear] on far right
- Uses `appStore.activeView` to highlight active tab
- Clicking tab sets `appStore.activeView`

**Step 4: Build StatusBar**

File: `src/lib/components/statusbar/StatusBar.svelte`

- Renders bottom bar with: git branch, model name, reasoning mode, token count, context
- All values from a `$state` status store (populated later by terminal output parsing)
- For now: show placeholder "–" values (they'll be filled when notification parsing is added)

**Step 5: Wire up App.svelte**

File: `src/App.svelte`

Layout structure:
- TopBar at top (fixed)
- Main area: Sidebar (left) + Content (right)
- StatusBar at bottom (fixed)
- Content switches between terminal view and insights view based on `appStore.activeView`

Use CSS Grid:
```css
.app {
  display: grid;
  grid-template-rows: 40px 1fr 28px;
  grid-template-columns: auto 1fr;
  height: 100vh;
}
```

**Step 6: Run `npm run tauri dev` and verify layout skeleton**

Expected: top bar, empty sidebar, empty content area, status bar.

**Step 7: Commit**

```bash
git add src/
git commit -m "feat: app shell with top bar, sidebar area, content area, status bar"
```

---

## Phase 5: Sidebar

### Task 6: Build the sidebar component

**Files:**
- Create: `src/lib/components/sidebar/Sidebar.svelte`
- Create: `src/lib/components/sidebar/WorkspaceItem.svelte`
- Create: `src/lib/components/sidebar/FolderItem.svelte`
- Create: `src/lib/components/sidebar/ContextMenu.svelte`

**Step 1: Build Sidebar.svelte**

- Resizable: mouse drag on right border changes `appStore.sidebarWidth`
- Minimizable: [◀] button toggles `appStore.sidebarMinimized`
- When minimized: 48px wide, shows only icons/initials + badges
- [+ New Workspace] button at top
- Lists folders (collapsible) with workspaces inside
- Lists ungrouped workspaces below
- [New Folder] button at bottom
- Background: var(--bg-sidebar)

**Step 2: Build WorkspaceItem.svelte**

- Props: workspace, isActive
- Shows workspace name, truncated
- Shows child sessions with:
  - Name
  - Notification badge (blue dot + count) if > 0
  - Status indicator color
- Click → sets active workspace
- Right-click → shows ContextMenu

**Step 3: Build FolderItem.svelte**

- Props: folder, workspaces (filtered)
- Collapsible header with folder name + arrow
- Lists WorkspaceItems inside
- Right-click → rename/delete context menu

**Step 4: Build ContextMenu.svelte**

- Generic right-click context menu component
- Props: items (array of {label, action}), position (x, y)
- Renders at mouse position
- Closes on click outside or Escape

**Step 5: Wire sidebar into App.svelte**

Render Sidebar in the left column of the grid.

**Step 6: Test — verify sidebar renders, resizes, minimizes**

**Step 7: Commit**

```bash
git add src/lib/components/sidebar/
git commit -m "feat: resizable minimizable sidebar with workspace and folder items"
```

---

## Phase 6: Terminal Split Layout

### Task 7: Build the split pane system

**Files:**
- Create: `src/lib/components/terminal/SplitContainer.svelte`
- Create: `src/lib/components/terminal/PaneHeader.svelte`
- Create: `src/lib/components/terminal/TerminalView.svelte`

**Step 1: Build SplitContainer.svelte (recursive)**

This is the core layout engine. Renders a `SplitNode` tree recursively:
- If `node.type === 'terminal'` → render TerminalPane
- If `node.type === 'split'` → render two children with a draggable divider
  - `direction: 'horizontal'` → children side by side
  - `direction: 'vertical'` → children stacked
- Divider is draggable to adjust `node.ratio` (default 0.5)
- Uses CSS flexbox with flex-basis based on ratio

**Step 2: Build PaneHeader.svelte**

- Small header bar on each terminal pane
- Shows: terminal name, bypass badge if applicable
- Buttons: [Split Horizontal ⊞], [Split Vertical ⊟], [Close ✕]
- Split buttons modify the workspace layout tree (wrap current node in a split, add new terminal)

**Step 3: Build TerminalView.svelte**

- Renders the active workspace's layout
- If no workspace active → show empty state with "Create a workspace to get started"
- Bottom bar: [🤖 New Workspace] and [+ Terminal] buttons
- [+ Terminal] adds a terminal to the active workspace by splitting the last pane

**Step 4: Wire TerminalView into App.svelte content area**

Render when `appStore.activeView === 'terminals'`.

**Step 5: Test — create a workspace programmatically with 4-grid layout, verify splits render and resize**

**Step 6: Commit**

```bash
git add src/lib/components/terminal/
git commit -m "feat: recursive split pane system with draggable dividers"
```

---

## Phase 7: New Workspace Modal

### Task 8: Build the workspace creation flow

**Files:**
- Create: `src/lib/components/workspace/NewWorkspaceModal.svelte`
- Create: `src/lib/components/workspace/LayoutPicker.svelte`
- Create: `src/lib/components/workspace/AgentPicker.svelte`
- Create: `src/lib/components/workspace/PathPicker.svelte`
- Create: `src/lib/stores/recent-paths.svelte.ts`
- Create: `src-tauri/src/commands_fs.rs` (for directory browsing)

**Step 1: Build PathPicker with recent paths**

File: `src/lib/stores/recent-paths.svelte.ts`

Class-based store:
- Loads/saves from `~/.config/gmux/recent-paths.json`
- Each entry: `{ path: string, frequency: number, lastUsed: string }`
- Score = frequency * recency_factor
- `addPath(path)` — increment frequency, update lastUsed
- `getPaths()` — sorted by score descending

File: `src/lib/components/workspace/PathPicker.svelte`

- Text input for path
- Browse button (uses Tauri dialog plugin for native folder picker)
- "Recent" dropdown button that shows recent paths sorted by score
- Clicking a recent path fills the input

File: `src-tauri/src/commands_fs.rs`

- `list_directory(path: String) -> Result<Vec<DirEntry>>` — for path autocomplete
- Register in lib.rs

**Step 2: Build LayoutPicker**

File: `src/lib/components/workspace/LayoutPicker.svelte`

- Grid of visual layout templates
- Each template is a button with a mini SVG/CSS preview of the layout
- Templates: Single (1), 2 Side-by-Side, 2 Vertical, 4 Grid, 3 Columns, 6 Grid
- Selected template highlighted with accent border
- Returns a `SplitNode` tree matching the selected template

**Step 3: Build AgentPicker**

File: `src/lib/components/workspace/AgentPicker.svelte`

- Agent rows: Claude Code, Codex CLI, Gemini CLI, Shell
- Each row: name, [−] count [+] buttons
- Claude row: extra checkbox "bypass permissions"
- Shows "X/Y slots assigned" at bottom
- Unassigned slots labeled as "auto-fill with shells"
- All counts default to 0 (plain terminals by default)

**Step 4: Build NewWorkspaceModal**

File: `src/lib/components/workspace/NewWorkspaceModal.svelte`

- Modal overlay (dark backdrop, centered card)
- Sections: Name input, PathPicker, LayoutPicker, AgentPicker
- [Cancel] and [🚀 Launch] buttons
- On Launch:
  1. Create workspace with chosen name, path, layout
  2. For each slot: determine shell command based on agent type
  3. Call Rust backend to spawn all PTYs in parallel
  4. Add workspace to appStore
  5. Set as active workspace
  6. Close modal
  7. Save recent path

**Step 5: Wire modal trigger from sidebar [+ New Workspace] button and bottom [🤖 New Workspace] button**

**Step 6: Test — open modal, select 4-grid, assign 2 Claude + 2 Shell, launch. Verify 4 terminals open instantly.**

**Step 7: Commit**

```bash
git add src/lib/components/workspace/ src/lib/stores/recent-paths.svelte.ts src-tauri/src/commands_fs.rs
git commit -m "feat: new workspace modal with layout picker, agent picker, recent paths"
```

---

## Phase 8: Parallel PTY Launch

### Task 9: Implement instant parallel terminal spawning

**Files:**
- Create: `src-tauri/src/commands_workspace.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create batch spawn command**

File: `src-tauri/src/commands_workspace.rs`

```rust
#[derive(serde::Deserialize)]
struct SpawnRequest {
    shell: String,
    cwd: String,
    command: Option<String>,
    cols: u16,
    rows: u16,
}

#[tauri::command]
async fn spawn_batch(
    requests: Vec<SpawnRequest>,
    channels: Vec<Channel<TerminalEvent>>,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<Vec<String>, String>
```

- Takes a list of spawn requests + corresponding channels
- Uses `tokio::spawn` for each request to spawn in parallel
- Each spawned task: create PTY, start reader thread on channel, optionally write command + '\r'
- Waits for all with `futures::future::join_all`
- Returns all PTY IDs

**Step 2: Register command in lib.rs**

**Step 3: Update frontend NewWorkspaceModal to use `spawn_batch`**

Instead of calling `create_pty` N times sequentially, build the batch request and call once.

**Step 4: Test — launch workspace with 6 Claude terminals, all should open in < 2 seconds**

**Step 5: Commit**

```bash
git add src-tauri/src/commands_workspace.rs src-tauri/src/lib.rs
git commit -m "feat: parallel batch PTY spawning for instant workspace launch"
```

---

## Phase 9: Session Persistence

### Task 10: Implement save/load for all state

**Files:**
- Create: `src-tauri/src/config/mod.rs`
- Create: `src/lib/stores/persistence.svelte.ts`
- Modify: `src/lib/stores/app.svelte.ts`

**Step 1: Rust config module**

File: `src-tauri/src/config/mod.rs`

- `get_config_dir() -> PathBuf` — returns `~/.config/gmux/`, creates if needed
- `save_state(filename: &str, data: &str) -> Result<()>` — write JSON to config dir
- `load_state(filename: &str) -> Result<Option<String>>` — read JSON from config dir

Tauri commands:
- `save_app_state(data: String) -> Result<()>`
- `load_app_state() -> Result<Option<String>>`

**Step 2: Persistence store**

File: `src/lib/stores/persistence.svelte.ts`

- `saveState()` — serializes appStore (workspaces, folders, sidebar state, recent paths) to JSON, calls Rust command
- `loadState()` — calls Rust command, deserializes, hydrates appStore
- `$effect` that auto-saves on any appStore change (debounced 2 seconds)

**Step 3: Hook into app lifecycle**

- On app mount: call `loadState()`
- On window close event (Tauri): call `saveState()`
- Debounced auto-save via `$effect`

**Step 4: Include terminal scrollback in persistence**

- When saving: for each terminal, call xterm's `buffer.active` to get scrollback content, save to `~/.config/gmux/scrollback/{terminalId}.txt`
- When loading: read scrollback file, write to terminal after mount

**Step 5: Test — create workspace, close app, reopen. Layout and scrollback should be restored.**

**Step 6: Commit**

```bash
git add src-tauri/src/config/ src/lib/stores/persistence.svelte.ts
git commit -m "feat: session persistence with auto-save and scrollback restoration"
```

---

## Phase 10: Notification System

### Task 11: Implement terminal output monitoring and notifications

**Files:**
- Create: `src/lib/utils/notification-detector.ts`
- Create: `src/lib/stores/notifications.svelte.ts`
- Modify: `src/lib/components/terminal/TerminalPane.svelte`
- Modify: `src/lib/components/sidebar/WorkspaceItem.svelte`

**Step 1: Build notification detector**

File: `src/lib/utils/notification-detector.ts`

- Takes terminal output chunks
- Maintains a line buffer (accumulates until newline)
- Tests each complete line against regex patterns
- Default patterns:
  - Claude: `/[❯›]\s*$/`, `/\?\s*\(y\/n\)/`, `/\bWaiting for/i`
  - Codex: `/\? for shortcuts/`, `/What should Codex do/`
  - Gemini: `/>>>\s*$/`
- Returns `{ matched: boolean, pattern: string }` for each line
- Patterns loaded from settings (configurable)

**Step 2: Build notifications store**

File: `src/lib/stores/notifications.svelte.ts`

Class-based store:
- Tracks per-session notification state
- `notify(sessionId, message)` — increments count, updates appStore session status to 'needs-input'
- `clear(sessionId)` — resets count to 0, status to 'running'
- Triggers desktop notification via Tauri notification plugin (if enabled in settings)

**Step 3: Wire into TerminalPane**

In the channel onmessage handler, pass each output chunk through the notification detector. If matched and the terminal is not focused, trigger notification.

When terminal receives focus, clear its notifications.

**Step 4: Update sidebar to show badges**

WorkspaceItem already reads `session.notificationCount` — verify blue dot + count renders.

**Step 5: Test — run `sleep 5 && echo "? (y/n)"` in a terminal, switch to another. Badge should appear after 5s.**

**Step 6: Commit**

```bash
git add src/lib/utils/notification-detector.ts src/lib/stores/notifications.svelte.ts
git commit -m "feat: notification system with pattern detection and desktop alerts"
```

---

## Phase 11: Statusbar — Live Terminal Info

### Task 12: Parse terminal output for model/context/token info

**Files:**
- Create: `src/lib/utils/status-parser.ts`
- Create: `src/lib/stores/status.svelte.ts`
- Modify: `src/lib/components/statusbar/StatusBar.svelte`

**Step 1: Build status parser**

File: `src/lib/utils/status-parser.ts`

Parses terminal output lines for:
- Model name: regex for `claude-opus-4`, `claude-sonnet`, `gpt-5`, `gemini` patterns
- Reasoning mode: regex for `reasoning:`, `reasoning_effort`, `high`, `medium`, `low`
- Token count: regex for `input:.*tokens`, `output:.*tokens`, or `↑` `↓` patterns
- Context: regex for `context:`, `ctx`, patterns like `64%`, `64k/200k`
- Git branch: gets from git2 backend for the active terminal's cwd

**Step 2: Build status store**

File: `src/lib/stores/status.svelte.ts`

- Holds current statusbar values (model, reasoning, tokens, context, branch)
- Updated by status parser when active terminal receives output
- Git branch updated by calling Rust command `get_git_branch(cwd)` when active terminal changes

**Step 3: Add git branch Tauri command**

File: `src-tauri/src/git/mod.rs`

- `get_current_branch(path: String) -> Result<Option<String>>` — uses git2 to open repo at path, return current branch name

**Step 4: Update StatusBar.svelte to read from status store**

**Step 5: Commit**

```bash
git add src/lib/utils/status-parser.ts src/lib/stores/status.svelte.ts src-tauri/src/git/
git commit -m "feat: live statusbar with model, tokens, context, git branch parsing"
```

---

## Phase 12: Insights — Usage Tab

### Task 13: Build the usage tracking view

**Files:**
- Create: `src-tauri/src/usage/mod.rs`
- Create: `src/lib/components/insights/InsightsView.svelte`
- Create: `src/lib/components/insights/UsageTab.svelte`
- Create: `src/lib/stores/usage.svelte.ts`

**Step 1: Rust usage parser**

File: `src-tauri/src/usage/mod.rs`

- `parse_claude_usage(home_dir: &str) -> Result<UsageData>` — reads Claude Code JSONL session files from `~/.claude/projects/*/sessions/*.jsonl`
- Parses each line for `usage` fields: `input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`
- Aggregates by: today, this week, this month
- Groups by session
- Returns structured data

Tauri command:
- `get_usage_data(period: String) -> Result<UsageData>` — period: "today" | "weekly" | "monthly"

**Step 2: Build usage store**

File: `src/lib/stores/usage.svelte.ts`

- Calls Rust command on refresh
- `$state` for current period and data
- `refresh()` method
- `setPeriod(period)` method

**Step 3: Build InsightsView.svelte**

- Container component that renders when `appStore.activeView === 'insights'`
- Left nav: Usage / Git / Info tabs (reads from `appStore.activeInsightsTab`)
- Right content: switches based on active tab

**Step 4: Build UsageTab.svelte**

- Period toggle: Today / Weekly / Monthly
- Token breakdown table: Input, Output, Cache Read, Cache Write + calculated costs
- Rate limits section with progress bars + reset countdown
  - Rate limit config loaded from settings
  - Progress bar width = percentage used
  - Reset time displayed as countdown
- Per-session breakdown list
- [Refresh] button

Cost calculation (approximate, from config):
- Input: $3/1M tokens (Claude), varies by provider
- Output: $15/1M tokens (Claude)
- Cache read: $0.30/1M tokens
- Cache write: $3.75/1M tokens

**Step 5: Wire into App.svelte — show InsightsView when activeView is 'insights'**

**Step 6: Test — verify real usage data appears from Claude session files**

**Step 7: Commit**

```bash
git add src-tauri/src/usage/ src/lib/components/insights/ src/lib/stores/usage.svelte.ts
git commit -m "feat: usage tracking tab with token breakdown, costs, rate limits"
```

---

## Phase 13: Insights — Git Tab

### Task 14: Build the git integration view

**Files:**
- Modify: `src-tauri/src/git/mod.rs`
- Create: `src/lib/components/insights/GitTab.svelte`
- Create: `src/lib/components/insights/DiffView.svelte`
- Create: `src/lib/stores/git.svelte.ts`

**Step 1: Expand Rust git module**

File: `src-tauri/src/git/mod.rs`

Tauri commands using git2:
- `get_branches(path: String) -> Result<Vec<BranchInfo>>` — list local branches, mark current
- `switch_branch(path: String, branch: String) -> Result<()>` — checkout branch
- `get_status(path: String) -> Result<Vec<FileStatus>>` — uncommitted changes with status (modified/added/deleted) and +/- line counts
- `get_diff(path: String, file: String) -> Result<Vec<DiffHunk>>` — unified diff for a specific file, returns hunks with old/new line numbers and content
- `stage_file(path: String, file: String) -> Result<()>`
- `unstage_file(path: String, file: String) -> Result<()>`
- `revert_file(path: String, file: String) -> Result<()>`

Each `DiffHunk`:
```rust
struct DiffLine {
    origin: char,  // '+', '-', ' '
    old_lineno: Option<u32>,
    new_lineno: Option<u32>,
    content: String,
}
```

**Step 2: Build git store**

File: `src/lib/stores/git.svelte.ts`

- `$state` for: branches, current branch, file statuses, selected file diff
- Methods: refresh, switchBranch, stageFile, unstageFile, revertFile, selectFile
- Uses active workspace's cwd as the git repo path

**Step 3: Build GitTab.svelte**

- Branch dropdown (select element) at top with current branch highlighted
- Click different branch → calls switchBranch
- Uncommitted changes list:
  - Each file: status icon (M/A/D/?), filename, +/- counts, clickable
  - [Stage All] and [Revert All] buttons
- Click on a file → shows DiffView for that file
- [Refresh] button

**Step 4: Build DiffView.svelte**

- Shows "← Back" button to return to file list
- File header: filename + total +/- counts
- Renders diff hunks:
  - Context lines: normal background
  - Added lines: green background (var(--diff-add))
  - Removed lines: red background (var(--diff-delete))
  - Line numbers on both sides (old and new)
- Monospace font, scrollable

**Step 5: Test — make changes in a git repo, verify status, diffs, branch switching work**

**Step 6: Commit**

```bash
git add src-tauri/src/git/ src/lib/components/insights/GitTab.svelte src/lib/components/insights/DiffView.svelte src/lib/stores/git.svelte.ts
git commit -m "feat: git tab with branch switching, uncommitted changes, inline diff view"
```

---

## Phase 14: Settings

### Task 15: Build the settings panel

**Files:**
- Create: `src/lib/components/settings/SettingsModal.svelte`
- Create: `src/lib/stores/settings.svelte.ts`
- Create: `src/lib/components/settings/AppearanceSettings.svelte`
- Create: `src/lib/components/settings/TerminalSettings.svelte`
- Create: `src/lib/components/settings/AiCliSettings.svelte`
- Create: `src/lib/components/settings/NotificationSettings.svelte`
- Create: `src/lib/components/settings/KeybindingSettings.svelte`
- Create: `src/lib/components/settings/RateLimitSettings.svelte`

**Step 1: Build settings store**

File: `src/lib/stores/settings.svelte.ts`

Class-based store, loaded from / saved to `~/.config/gmux/settings.json`:

```typescript
class SettingsStore {
  appearance = $state({
    accentColor: '#10a37f',
    fontUi: 'Inter, system-ui, sans-serif',
    fontCode: '"JetBrains Mono", monospace',
    fontSize: 14,
  });
  terminal = $state({
    defaultShell: '',  // detected from $SHELL on first run
    scrollbackLines: 10000,
    cursorStyle: 'block' as 'block' | 'beam' | 'underline',
  });
  aiClis = $state({
    claude: { path: 'claude', enabled: true },
    codex: { path: 'codex', enabled: true },
    gemini: { path: 'gemini', enabled: true },
    custom: [] as Array<{ name: string, command: string }>,
  });
  rateLimits = $state({
    claude: { fiveHourWindow: 1000000, weeklyLimit: 5000000, resetDay: 'monday', resetHour: 0 },
    codex: { fiveHourWindow: 0, weeklyLimit: 0, resetDay: 'monday', resetHour: 0 },
    gemini: { fiveHourWindow: 0, weeklyLimit: 0, resetDay: 'monday', resetHour: 0 },
  });
  notifications = $state({
    desktopEnabled: true,
    soundEnabled: false,
    customPatterns: [] as string[],
  });
  keybindings = $state({
    splitHorizontal: 'Ctrl+Shift+D',
    splitVertical: 'Ctrl+Shift+R',
    closePane: 'Ctrl+Shift+W',
    newWorkspace: 'Ctrl+Shift+N',
    newTerminal: 'Ctrl+Shift+T',
    toggleSidebar: 'Ctrl+B',
    search: 'Ctrl+Shift+F',
    nextPane: 'Ctrl+Tab',
    prevPane: 'Ctrl+Shift+Tab',
  });

  async load() { /* load from ~/.config/gmux/settings.json via Rust command */ }
  async save() { /* save to file */ }
}

export const settingsStore = new SettingsStore();
```

**Step 2: Build SettingsModal**

Modal with left nav tabs: Appearance, Terminal, AI CLIs, Rate Limits, Notifications, Keybindings.
Each tab renders corresponding settings component.

**Step 3: Build each settings section component**

Each section renders form fields matching the settings store properties.
On change → update store → trigger save.

- AppearanceSettings: color picker for accent, font dropdowns, font size slider
- TerminalSettings: shell input (with auto-detect), scrollback number input, cursor style radio buttons
- AiCliSettings: path inputs for each CLI, enabled toggles, custom command list
- RateLimitSettings: per provider inputs for window size, weekly limit, reset schedule
- NotificationSettings: toggle switches, custom regex pattern list
- KeybindingSettings: list of actions with editable shortcut inputs (record keypress)

**Step 4: Wire settings gear icon in TopBar to open SettingsModal**

**Step 5: Apply settings reactively**

- Font/size changes update CSS variables and xterm terminal options
- Shell changes affect new terminal spawns
- Notification patterns update the detector

**Step 6: Test — change font size, verify terminals update. Change accent color, verify UI updates.**

**Step 7: Commit**

```bash
git add src/lib/components/settings/ src/lib/stores/settings.svelte.ts
git commit -m "feat: settings panel with appearance, terminal, AI CLI, notifications, keybindings"
```

---

## Phase 15: Insights — Info Tab

### Task 16: Build the info tab

**Files:**
- Create: `src/lib/components/insights/InfoTab.svelte`

**Step 1: Build InfoTab.svelte**

Displays:
- App version (from package.json via Tauri)
- OS info (from Tauri os-info plugin or Rust std::env)
- Configured AI CLIs with their paths and whether they're found on $PATH
- Active sessions list with status

Tauri command:
- `get_system_info() -> Result<SystemInfo>` — returns OS, arch, hostname
- `check_cli_exists(command: String) -> Result<bool>` — runs `which` to verify CLI exists

**Step 2: Wire into InsightsView**

**Step 3: Commit**

```bash
git add src/lib/components/insights/InfoTab.svelte
git commit -m "feat: info tab with system info and CLI status"
```

---

## Phase 16: Keyboard Shortcuts

### Task 17: Implement global keyboard shortcuts

**Files:**
- Create: `src/lib/utils/keybindings.ts`
- Modify: `src/App.svelte`

**Step 1: Build keybinding handler**

File: `src/lib/utils/keybindings.ts`

- Registers global keydown listener on window
- Matches against `settingsStore.keybindings`
- Actions: splitHorizontal, splitVertical, closePane, newWorkspace, newTerminal, toggleSidebar, search, nextPane, prevPane
- Each action calls the appropriate store method or component function

**Step 2: Wire into App.svelte**

Initialize keybinding handler on mount, dispose on unmount.

**Step 3: Test — Ctrl+Shift+N opens new workspace modal, Ctrl+B toggles sidebar, etc.**

**Step 4: Commit**

```bash
git add src/lib/utils/keybindings.ts
git commit -m "feat: configurable keyboard shortcuts"
```

---

## Phase 17: Polish & Final Integration

### Task 18: Final integration, edge cases, and polish

**Files:** Various (all components)

**Step 1: Terminal tab support within panes**

- Each pane can have multiple terminals (tabs at top of pane)
- Tab bar: shows terminal names, click to switch, [+] to add, [✕] to close
- Active tab content rendered, others hidden but PTY still running

**Step 2: Drag & drop in sidebar**

- Workspaces draggable between folders
- Sessions draggable to reorder within workspace
- Use HTML5 drag and drop API

**Step 3: Window state persistence**

- Save window size/position on close (Tauri window event)
- Restore on next launch

**Step 4: Error handling for PTY failures**

- If PTY spawn fails (e.g., CLI not found), show error in terminal pane: "Failed to start: command not found"
- Don't crash the app

**Step 5: Handle terminal resize on split changes**

- When split ratios change or panes are added/removed, trigger fitAddon.fit() + resize_pty for all affected terminals

**Step 6: Full test of the complete flow**

1. Launch app → empty state
2. Create workspace "My Project" with 4-grid, 2 Claude (bypass) + 2 Shell
3. All 4 terminals open instantly
4. Verify Claude runs with --dangerously-skip-permissions
5. Switch to Insights → Usage, verify real data
6. Switch to Git, verify branches and changes
7. Close app, reopen → layout and scrollback restored
8. Verify notifications work (trigger input prompt)

**Step 7: Commit**

```bash
git add -A
git commit -m "feat: polish — tabs in panes, drag-drop, window persistence, error handling"
```

---

## Task Dependencies

```
Task 1 (scaffold)
  └─> Task 2 (PTY manager)
       └─> Task 3 (Tauri commands)
            └─> Task 4 (terminal component)
                 └─> Task 5 (app shell)
                      ├─> Task 6 (sidebar)
                      └─> Task 7 (split panes)
                           └─> Task 8 (new workspace modal)
                                └─> Task 9 (parallel spawn)
                                     └─> Task 10 (persistence)
                                          ├─> Task 11 (notifications)
                                          │    └─> Task 12 (statusbar parsing)
                                          ├─> Task 13 (usage tab)
                                          ├─> Task 14 (git tab)
                                          ├─> Task 15 (settings)
                                          │    └─> Task 17 (keybindings)
                                          └─> Task 16 (info tab)
                                               └─> Task 18 (polish)
```
