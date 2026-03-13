import type { Workspace, Folder, AppView, InsightsTab, TerminalSession } from '$lib/types/workspace';

class AppStore {
	workspaces = $state<Workspace[]>([]);
	folders = $state<Folder[]>([]);
	activeWorkspaceId = $state<string | null>(null);
	activeView = $state<AppView>('terminals');
	activeInsightsTab = $state<InsightsTab>('usage');
	sidebarWidth = $state(250);
	sidebarMinimized = $state(false);
	activeTerminalId = $state<string | null>(null);
	showNewWorkspaceModal = $state(false);
	showSettings = $state(false);

	get activeWorkspace(): Workspace | undefined {
		return this.workspaces.find((w) => w.id === this.activeWorkspaceId);
	}

	get workspacesByFolder(): Map<string, Workspace[]> {
		const map = new Map<string, Workspace[]>();
		for (const ws of this.workspaces) {
			if (ws.folderId) {
				const list = map.get(ws.folderId) ?? [];
				list.push(ws);
				map.set(ws.folderId, list);
			}
		}
		return map;
	}

	get ungroupedWorkspaces(): Workspace[] {
		return this.workspaces.filter((w) => !w.folderId);
	}

	addWorkspace(ws: Workspace) {
		this.workspaces.push(ws);
		this.activeWorkspaceId = ws.id;
	}

	removeWorkspace(id: string) {
		this.workspaces = this.workspaces.filter((w) => w.id !== id);
		if (this.activeWorkspaceId === id) {
			this.activeWorkspaceId = this.workspaces[0]?.id ?? null;
		}
	}

	renameWorkspace(id: string, name: string) {
		const ws = this.workspaces.find((w) => w.id === id);
		if (ws) ws.name = name;
	}

	moveWorkspaceToFolder(workspaceId: string, folderId: string | undefined) {
		const ws = this.workspaces.find((w) => w.id === workspaceId);
		if (ws) ws.folderId = folderId;
	}

	setActiveWorkspace(id: string) {
		this.activeWorkspaceId = id;
	}

	addFolder(name: string) {
		this.folders.push({ id: crypto.randomUUID(), name, collapsed: false });
	}

	renameFolder(id: string, name: string) {
		const f = this.folders.find((f) => f.id === id);
		if (f) f.name = name;
	}

	removeFolder(id: string) {
		this.folders = this.folders.filter((f) => f.id !== id);
		this.workspaces
			.filter((w) => w.folderId === id)
			.forEach((w) => (w.folderId = undefined));
	}

	toggleFolder(id: string) {
		const f = this.folders.find((f) => f.id === id);
		if (f) f.collapsed = !f.collapsed;
	}

	toggleSidebar() {
		this.sidebarMinimized = !this.sidebarMinimized;
	}

	addSessionToWorkspace(workspaceId: string, session: TerminalSession) {
		const ws = this.workspaces.find((w) => w.id === workspaceId);
		if (ws) ws.sessions.push(session);
	}

	updateSessionStatus(sessionId: string, status: TerminalSession['status']) {
		for (const ws of this.workspaces) {
			const s = ws.sessions.find((s) => s.id === sessionId);
			if (s) {
				s.status = status;
				break;
			}
		}
	}

	incrementNotification(sessionId: string) {
		for (const ws of this.workspaces) {
			const s = ws.sessions.find((s) => s.id === sessionId);
			if (s) {
				s.notificationCount++;
				break;
			}
		}
	}

	clearNotification(sessionId: string) {
		for (const ws of this.workspaces) {
			const s = ws.sessions.find((s) => s.id === sessionId);
			if (s) {
				s.notificationCount = 0;
				s.status = 'running';
				break;
			}
		}
	}
}

export const appStore = new AppStore();
