import { parseStatusLine } from '$lib/utils/status-parser';
import { invoke } from '@tauri-apps/api/core';

const GIT_BRANCH_DEBOUNCE_MS = 2000;

class StatusStore {
	model = $state('\u2013');
	reasoning = $state('\u2013');
	inputTokens = $state(0);
	outputTokens = $state(0);
	contextPercent = $state(0);
	contextUsed = $state(0);
	contextTotal = $state(0);
	gitBranch = $state('\u2013');
	private lastBranchQuery = 0;
	private lastBranchCwd = '';

	updateFromLine(line: string) {
		const info = parseStatusLine(line);
		if (info.model) this.model = info.model;
		if (info.reasoning) this.reasoning = info.reasoning;
		if (info.inputTokens !== undefined) this.inputTokens = info.inputTokens;
		if (info.outputTokens !== undefined) this.outputTokens = info.outputTokens;
		if (info.contextPercent !== undefined) this.contextPercent = info.contextPercent;
		if (info.contextUsed !== undefined) this.contextUsed = info.contextUsed;
		if (info.contextTotal !== undefined) this.contextTotal = info.contextTotal;
	}

	async updateGitBranch(cwd: string) {
		const now = Date.now();
		if (cwd === this.lastBranchCwd && now - this.lastBranchQuery < GIT_BRANCH_DEBOUNCE_MS) {
			return;
		}
		this.lastBranchQuery = now;
		this.lastBranchCwd = cwd;
		try {
			const branch = await invoke<string | null>('get_current_branch', { path: cwd });
			if (branch) this.gitBranch = branch;
		} catch (e) {
			console.error('Failed to update git branch:', e);
			this.gitBranch = '\u2013';
		}
	}

	reset() {
		this.model = '\u2013';
		this.reasoning = '\u2013';
		this.inputTokens = 0;
		this.outputTokens = 0;
		this.contextPercent = 0;
		this.contextUsed = 0;
		this.contextTotal = 0;
		this.gitBranch = '\u2013';
		this.lastBranchQuery = 0;
		this.lastBranchCwd = '';
	}
}

export const statusStore = new StatusStore();
