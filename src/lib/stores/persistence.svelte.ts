import { invoke } from '@tauri-apps/api/core';
import { appStore } from './app.svelte';
import type { Workspace, Folder } from '$lib/types/workspace';

interface WindowState {
	width: number;
	height: number;
	x: number;
	y: number;
}

export interface PathEntry {
	path: string;
	frequency: number;
	lastUsed: string;
}

interface PersistedState {
	workspaces: Workspace[];
	folders: Folder[];
	sidebarWidth: number;
	sidebarMinimized: boolean;
	activeWorkspaceId: string | null;
	windowState?: WindowState;
	recentPaths?: PathEntry[];
}

class PersistenceStore {
	private saveTimeout: ReturnType<typeof setTimeout> | null = null;
	private saveQueue: Promise<void> = Promise.resolve();
	windowState: WindowState | null = null;
	recentPaths: PathEntry[] = [];

	private enqueue(fn: () => Promise<void>): Promise<void> {
		this.saveQueue = this.saveQueue.then(fn, fn);
		return this.saveQueue;
	}

	async saveState(windowState?: WindowState) {
		return this.enqueue(async () => {
			const state: PersistedState = {
				workspaces: appStore.workspaces.map((ws) => ({
					...ws,
					sessions: ws.sessions.map((s) => ({ ...s, notificationCount: 0 }))
				})),
				folders: appStore.folders,
				sidebarWidth: appStore.sidebarWidth,
				sidebarMinimized: appStore.sidebarMinimized,
				activeWorkspaceId: appStore.activeWorkspaceId,
				recentPaths: this.recentPaths
			};

			if (windowState) {
				state.windowState = windowState;
				this.windowState = windowState;
			} else if (this.windowState) {
				state.windowState = this.windowState;
			}

			await invoke('save_app_state', { data: JSON.stringify(state) });
		});
	}

	private validateState(parsed: unknown): PersistedState | null {
		if (!parsed || typeof parsed !== 'object') return null;
		const obj = parsed as Record<string, unknown>;
		if (!Array.isArray(obj.workspaces)) return null;
		for (const ws of obj.workspaces) {
			if (!ws || typeof ws !== 'object') return null;
			const w = ws as Record<string, unknown>;
			if (typeof w.id !== 'string' || typeof w.name !== 'string') return null;
			if (!Array.isArray(w.sessions)) return null;
			for (const s of w.sessions) {
				if (!s || typeof s !== 'object') return null;
				const sess = s as Record<string, unknown>;
				if (typeof sess.id !== 'string' || typeof sess.name !== 'string') return null;
			}
			if (!w.layout || typeof w.layout !== 'object') return null;
		}
		if (obj.folders !== undefined && !Array.isArray(obj.folders)) return null;
		return parsed as PersistedState;
	}

	async loadState() {
		const raw = await invoke<string | null>('load_app_state');
		if (!raw) return;

		let parsed: unknown;
		try {
			parsed = JSON.parse(raw);
		} catch {
			return;
		}

		const state = this.validateState(parsed);
		if (!state) return;

		if (state.workspaces) {
			appStore.setWorkspaces(state.workspaces.map((ws) => ({
				...ws,
				sessions: ws.sessions.map((s) => ({
					...s,
					ptyId: undefined,
					status: 'ready' as const,
					notificationCount: 0
				}))
			})));
		}

		if (state.folders) {
			appStore.folders = state.folders;
		}

		if (state.sidebarWidth != null) {
			appStore.sidebarWidth = state.sidebarWidth;
		}

		if (state.sidebarMinimized != null) {
			appStore.sidebarMinimized = state.sidebarMinimized;
		}

		if (state.activeWorkspaceId) {
			appStore.activeWorkspaceId = state.activeWorkspaceId;
		}

		if (state.windowState) {
			this.windowState = state.windowState;
		}

		if (state.recentPaths && Array.isArray(state.recentPaths)) {
			this.recentPaths = state.recentPaths;
		}
	}

	scheduleSave() {
		if (this.saveTimeout) clearTimeout(this.saveTimeout);
		this.saveTimeout = setTimeout(() => this.saveState(), 2000);
	}
}

export const persistence = new PersistenceStore();
