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
	version = $state(0);

	private sessionIndex = new Map<string, { workspaceId: string; session: TerminalSession }>();

	private bump() {
		this.version++;
	}

	private rebuildSessionIndex() {
		this.sessionIndex.clear();
		for (const ws of this.workspaces) {
			for (const s of ws.sessions) {
				this.sessionIndex.set(s.id, { workspaceId: ws.id, session: s });
			}
		}
	}

	private indexSession(workspaceId: string, session: TerminalSession) {
		this.sessionIndex.set(session.id, { workspaceId, session });
	}

	private unindexSession(sessionId: string) {
		this.sessionIndex.delete(sessionId);
	}

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
		for (const s of ws.sessions) {
			this.indexSession(ws.id, s);
		}
		this.activeWorkspaceId = ws.id;
		this.bump();
	}

	removeWorkspace(id: string) {
		const ws = this.workspaces.find((w) => w.id === id);
		if (ws) {
			for (const s of ws.sessions) {
				this.unindexSession(s.id);
			}
		}
		this.workspaces = this.workspaces.filter((w) => w.id !== id);
		if (this.activeWorkspaceId === id) {
			this.activeWorkspaceId = this.workspaces[0]?.id ?? null;
		}
		this.bump();
		return ws?.sessions.map((s) => s.id) ?? [];
	}

	renameWorkspace(id: string, name: string) {
		const ws = this.workspaces.find((w) => w.id === id);
		if (ws) {
			ws.name = name;
			this.bump();
		}
	}

	moveWorkspaceToFolder(workspaceId: string, folderId: string | undefined) {
		const ws = this.workspaces.find((w) => w.id === workspaceId);
		if (ws) {
			ws.folderId = folderId;
			this.bump();
		}
	}

	setActiveWorkspace(id: string) {
		this.activeWorkspaceId = id;
		this.bump();
	}

	addFolder(name: string) {
		this.folders.push({ id: crypto.randomUUID(), name, collapsed: false });
		this.bump();
	}

	renameFolder(id: string, name: string) {
		const f = this.folders.find((f) => f.id === id);
		if (f) {
			f.name = name;
			this.bump();
		}
	}

	removeFolder(id: string) {
		this.folders = this.folders.filter((f) => f.id !== id);
		this.workspaces
			.filter((w) => w.folderId === id)
			.forEach((w) => (w.folderId = undefined));
		this.bump();
	}

	toggleFolder(id: string) {
		const f = this.folders.find((f) => f.id === id);
		if (f) {
			f.collapsed = !f.collapsed;
		}
	}

	toggleSidebar() {
		this.sidebarMinimized = !this.sidebarMinimized;
	}

	addSessionToWorkspace(workspaceId: string, session: TerminalSession) {
		const ws = this.workspaces.find((w) => w.id === workspaceId);
		if (ws) {
			ws.sessions.push(session);
			this.indexSession(workspaceId, session);
			this.bump();
		}
	}

	removeSessionFromWorkspace(workspaceId: string, sessionId: string) {
		const ws = this.workspaces.find((w) => w.id === workspaceId);
		if (ws) {
			ws.sessions = ws.sessions.filter((s) => s.id !== sessionId);
			this.unindexSession(sessionId);
			this.bump();
		}
	}

	updateSessionStatus(sessionId: string, status: TerminalSession['status']) {
		const entry = this.sessionIndex.get(sessionId);
		if (entry) {
			entry.session.status = status;
			this.bump();
		}
	}

	incrementNotification(sessionId: string) {
		const entry = this.sessionIndex.get(sessionId);
		if (entry) {
			entry.session.notificationCount++;
		}
	}

	clearNotification(sessionId: string) {
		const entry = this.sessionIndex.get(sessionId);
		if (entry) {
			entry.session.notificationCount = 0;
			if (entry.session.status === 'needs-input') {
				entry.session.status = 'running';
			}
		}
	}

	setWorkspaces(workspaces: Workspace[]) {
		this.workspaces = workspaces;
		this.rebuildSessionIndex();
	}
}

export const appStore = new AppStore();
