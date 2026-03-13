import { persistence } from './persistence.svelte';

interface PathEntry {
	path: string;
	frequency: number;
	lastUsed: string;
}

const MAX_PATHS = 10;

class RecentPathsStore {
	get entries(): PathEntry[] {
		return persistence.recentPaths;
	}

	private score(entry: PathEntry): number {
		const daysSinceLastUse =
			(Date.now() - new Date(entry.lastUsed).getTime()) / (1000 * 60 * 60 * 24);
		const recencyFactor = 1 / (1 + daysSinceLastUse * 0.1);
		return entry.frequency * recencyFactor;
	}

	addPath(path: string) {
		const existing = persistence.recentPaths.find((e) => e.path === path);
		if (existing) {
			existing.frequency++;
			existing.lastUsed = new Date().toISOString();
		} else {
			persistence.recentPaths.push({
				path,
				frequency: 1,
				lastUsed: new Date().toISOString()
			});
		}
		persistence.scheduleSave();
	}

	getPaths(): PathEntry[] {
		return [...persistence.recentPaths].sort((a, b) => this.score(b) - this.score(a)).slice(0, MAX_PATHS);
	}

	removePath(path: string) {
		persistence.recentPaths = persistence.recentPaths.filter((e) => e.path !== path);
		persistence.scheduleSave();
	}
}

export const recentPathsStore = new RecentPathsStore();
export type { PathEntry };
