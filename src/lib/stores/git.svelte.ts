import { invoke } from '@tauri-apps/api/core';

export interface BranchInfo {
	name: string;
	isCurrent: boolean;
}

export interface FileStatus {
	path: string;
	status: string;
	additions: number;
	deletions: number;
}

export interface DiffLineInfo {
	origin: string;
	oldLineno: number | null;
	newLineno: number | null;
	content: string;
}

export interface FileDiff {
	path: string;
	hunks: DiffLineInfo[][];
}

class GitStore {
	branches = $state<BranchInfo[]>([]);
	currentBranch = $state('');
	files = $state<FileStatus[]>([]);
	selectedFile = $state<string | null>(null);
	fileDiff = $state<FileDiff | null>(null);
	loading = $state(false);
	repoPath = $state('');

	async refresh() {
		if (!this.repoPath) return;
		this.loading = true;
		try {
			[this.branches, this.files] = await Promise.all([
				invoke<BranchInfo[]>('get_branches', { path: this.repoPath }),
				invoke<FileStatus[]>('get_git_status', { path: this.repoPath })
			]);
			this.currentBranch = this.branches.find((b) => b.isCurrent)?.name ?? '';
		} catch {
			this.branches = [];
			this.files = [];
			this.currentBranch = '';
		}
		this.loading = false;
	}

	async switchBranch(branch: string) {
		if (!this.repoPath || branch === this.currentBranch) return;
		try {
			await invoke('switch_branch', { path: this.repoPath, branch });
			this.clearSelection();
			await this.refresh();
		} catch (e) {
			console.error('Failed to switch branch:', e);
		}
	}

	async selectFile(file: string) {
		this.selectedFile = file;
		try {
			this.fileDiff = await invoke<FileDiff>('get_file_diff', {
				path: this.repoPath,
				file
			});
		} catch {
			this.fileDiff = null;
		}
	}

	async stageFile(file: string) {
		try {
			await invoke('stage_file', { path: this.repoPath, file });
			await this.refresh();
		} catch (e) {
			console.error('Failed to stage file:', e);
		}
	}

	async revertFile(file: string) {
		try {
			await invoke('revert_file', { path: this.repoPath, file });
			this.clearSelection();
			await this.refresh();
		} catch (e) {
			console.error('Failed to revert file:', e);
		}
	}

	clearSelection() {
		this.selectedFile = null;
		this.fileDiff = null;
	}

	setRepoPath(path: string) {
		if (this.repoPath === path) return;
		this.repoPath = path;
		this.clearSelection();
		this.refresh();
	}
}

export const gitStore = new GitStore();
