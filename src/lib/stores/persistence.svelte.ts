import { invoke } from '@tauri-apps/api/core';
import { appStore } from './app.svelte';
import type { Workspace, Folder } from '$lib/types/workspace';

interface WindowState {
	width: number;
	height: number;
	x: number;
	y: number;
}

interface PersistedState {
	workspaces: Workspace[];
	folders: Folder[];
	sidebarWidth: number;
	sidebarMinimized: boolean;
	activeWorkspaceId: string | null;
	windowState?: WindowState;
	recentPaths?: unknown[];
}

class PersistenceStore {
	private saveTimeout: ReturnType<typeof setTimeout> | null = null;
	windowState: WindowState | null = null;

	async saveState(windowState?: WindowState) {
		const raw = await invoke<string | null>('load_app_state');
		let existing: Record<string, unknown> = {};
		if (raw) {
			try {
				existing = JSON.parse(raw);
			} catch (e) {
				console.error('Failed to parse existing state:', e);
			}
		}

		const state: PersistedState = {
			...existing,
			workspaces: appStore.workspaces.map((ws) => ({
				...ws,
				sessions: ws.sessions.map((s) => ({ ...s, notificationCount: 0 }))
			})),
			folders: appStore.folders,
			sidebarWidth: appStore.sidebarWidth,
			sidebarMinimized: appStore.sidebarMinimized,
			activeWorkspaceId: appStore.activeWorkspaceId
		};

		if (windowState) {
			state.windowState = windowState;
		}

		await invoke('save_app_state', { data: JSON.stringify(state) });
	}

	async loadState() {
		const raw = await invoke<string | null>('load_app_state');
		if (!raw) return;
		const state: PersistedState = JSON.parse(raw);

		if (state.workspaces) {
			appStore.workspaces = state.workspaces.map((ws) => ({
				...ws,
				sessions: ws.sessions.map((s) => ({
					...s,
					status: 'exited' as const,
					notificationCount: 0
				}))
			}));
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
	}

	scheduleSave() {
		if (this.saveTimeout) clearTimeout(this.saveTimeout);
		this.saveTimeout = setTimeout(() => this.saveState(), 2000);
	}
}

export const persistence = new PersistenceStore();
