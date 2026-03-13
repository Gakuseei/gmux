import { invoke } from '@tauri-apps/api/core';
import { appStore } from './app.svelte';
import type { Workspace, Folder } from '$lib/types/workspace';

interface PersistedState {
	workspaces: Workspace[];
	folders: Folder[];
	sidebarWidth: number;
	sidebarMinimized: boolean;
	activeWorkspaceId: string | null;
}

class PersistenceStore {
	private saveTimeout: ReturnType<typeof setTimeout> | null = null;

	async saveState() {
		const state: PersistedState = {
			workspaces: appStore.workspaces.map((ws) => ({
				...ws,
				sessions: ws.sessions.map((s) => ({ ...s, notificationCount: 0 }))
			})),
			folders: appStore.folders,
			sidebarWidth: appStore.sidebarWidth,
			sidebarMinimized: appStore.sidebarMinimized,
			activeWorkspaceId: appStore.activeWorkspaceId
		};
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
	}

	scheduleSave() {
		if (this.saveTimeout) clearTimeout(this.saveTimeout);
		this.saveTimeout = setTimeout(() => this.saveState(), 2000);
	}
}

export const persistence = new PersistenceStore();
