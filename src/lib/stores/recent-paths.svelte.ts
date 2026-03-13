interface PathEntry {
	path: string;
	frequency: number;
	lastUsed: string;
}

const STORAGE_KEY = 'gmux-recent-paths';
const MAX_PATHS = 10;

class RecentPathsStore {
	entries = $state<PathEntry[]>([]);

	constructor() {
		this.load();
	}

	private load() {
		try {
			const raw = localStorage.getItem(STORAGE_KEY);
			if (raw) {
				this.entries = JSON.parse(raw);
			}
		} catch {
			this.entries = [];
		}
	}

	private save() {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(this.entries));
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
