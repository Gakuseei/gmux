import { invoke } from '@tauri-apps/api/core';

interface PathEntry {
	path: string;
	frequency: number;
	lastUsed: string;
}

const FILENAME = 'recent-paths.json';
const MAX_PATHS = 10;

class RecentPathsStore {
	entries = $state<PathEntry[]>([]);

	constructor() {
		this.load();
	}

	private async load() {
		try {
			const raw = await invoke<string | null>('load_app_state');
			if (raw) {
				const parsed = JSON.parse(raw);
				if (parsed.recentPaths && Array.isArray(parsed.recentPaths)) {
					this.entries = parsed.recentPaths;
				}
			}
		} catch (e) {
			console.error('Failed to load recent paths:', e);
			this.entries = [];
		}
	}

	private async save() {
		try {
			const existing = await invoke<string | null>('load_app_state');
			let state: Record<string, unknown> = {};
			if (existing) {
				state = JSON.parse(existing);
			}
			state.recentPaths = this.entries;
			await invoke('save_app_state', { data: JSON.stringify(state) });
		} catch (e) {
			console.error('Failed to save recent paths:', e);
		}
	}

	private score(entry: PathEntry): number {
		const daysSinceLastUse =
			(Date.now() - new Date(entry.lastUsed).getTime()) / (1000 * 60 * 60 * 24);
		const recencyFactor = 1 / (1 + daysSinceLastUse * 0.1);
		return entry.frequency * recencyFactor;
	}

	addPath(path: string) {
		const existing = this.entries.find((e) => e.path === path);
		if (existing) {
			existing.frequency++;
			existing.lastUsed = new Date().toISOString();
		} else {
			this.entries.push({
				path,
				frequency: 1,
				lastUsed: new Date().toISOString()
			});
		}
		this.save();
	}

	getPaths(): PathEntry[] {
		return [...this.entries].sort((a, b) => this.score(b) - this.score(a)).slice(0, MAX_PATHS);
	}

	removePath(path: string) {
		this.entries = this.entries.filter((e) => e.path !== path);
		this.save();
	}
}

export const recentPathsStore = new RecentPathsStore();
export type { PathEntry };
