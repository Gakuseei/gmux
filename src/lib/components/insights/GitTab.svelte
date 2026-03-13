<script lang="ts">
	import { gitStore } from '$lib/stores/git.svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';
	import DiffView from './DiffView.svelte';

	let errorMessage = $state<string | null>(null);

	async function withError(label: string, fn: () => Promise<void>) {
		errorMessage = null;
		try {
			await fn();
		} catch (e) {
			errorMessage = `${label}: ${String(e)}`;
		}
	}

	function handleStageFile(path: string) {
		withError(`Failed to stage ${path}`, () => gitStore.stageFile(path));
	}

	function handleUnstageFile(path: string) {
		withError(`Failed to unstage ${path}`, () => gitStore.unstageFile(path));
	}

	async function handleRevertFile(path: string) {
		const confirmed = await confirm(`Revert ${path}? This cannot be undone.`, {
			title: 'Confirm Revert',
			kind: 'warning'
		});
		if (!confirmed) return;
		withError(`Failed to revert ${path}`, () => gitStore.revertFile(path));
	}

	function handleStageAll() {
		withError('Failed to stage all', async () => {
			for (const f of gitStore.files) await gitStore.stageFile(f.path);
		});
	}

	function statusIcon(status: string): string {
		switch (status) {
			case 'modified':
				return 'M';
			case 'added':
				return 'A';
			case 'deleted':
				return 'D';
			default:
				return '?';
		}
	}

	function statusClass(status: string): string {
		switch (status) {
			case 'modified':
				return 'status-modified';
			case 'added':
				return 'status-added';
			case 'deleted':
				return 'status-deleted';
			default:
				return 'status-untracked';
		}
	}
</script>

{#if gitStore.fileDiff}
	<DiffView diff={gitStore.fileDiff} onBack={() => gitStore.clearSelection()} />
{:else if !gitStore.repoPath}
	<div class="empty-state">No workspace selected</div>
{:else}
	<div class="git-tab">
		{#if errorMessage}
			<div class="error-banner">
				<span>{errorMessage}</span>
				<button class="error-dismiss" onclick={() => (errorMessage = null)} aria-label="Dismiss error">&times;</button>
			</div>
		{/if}
		<div class="toolbar">
			<div class="branch-select-wrapper">
				<select
					class="branch-select"
					value={gitStore.currentBranch}
					onchange={(e) => gitStore.switchBranch(e.currentTarget.value)}
					disabled={gitStore.loading}
				>
					{#each gitStore.branches as branch}
						<option value={branch.name}>
							{branch.name}{branch.isCurrent ? ' *' : ''}
						</option>
					{/each}
				</select>
			</div>
			<button
				class="refresh-btn"
				aria-label="Refresh"
				onclick={() => gitStore.refresh()}
				disabled={gitStore.loading}
			>
				<svg
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class:spinning={gitStore.loading}
				>
					<path d="M21 12a9 9 0 1 1-6.219-8.56" />
					<polyline points="21 3 21 9 15 9" />
				</svg>
			</button>
		</div>

		<div class="changes-section">
			<div class="section-header">
				<span class="section-title">
					UNCOMMITTED CHANGES ({gitStore.files.length})
				</span>
				{#if gitStore.files.length > 0}
					<div class="section-actions">
						<button
							class="action-btn"
							onclick={handleStageAll}
						>
							Stage All
						</button>
					</div>
				{/if}
			</div>

			{#if gitStore.files.length === 0 && !gitStore.loading}
				<div class="empty-changes">No uncommitted changes</div>
			{/if}

			<div class="file-list">
				{#each gitStore.files as file}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="file-item"
						role="button"
						tabindex="0"
						onclick={() => gitStore.selectFile(file.path)}
						onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') gitStore.selectFile(file.path); }}
					>
						<span class="file-status {statusClass(file.status)}">
							{statusIcon(file.status)}
						</span>
						<span class="file-path">{file.path}</span>
						<span class="file-stats">
							{#if file.additions > 0}
								<span class="stat-add">+{file.additions}</span>
							{/if}
							{#if file.deletions > 0}
								<span class="stat-del">-{file.deletions}</span>
							{/if}
						</span>
						<span class="file-actions">
							<button
								class="icon-action"
								title="Stage"
								onclick={(e: MouseEvent) => { e.stopPropagation(); handleStageFile(file.path); }}
							>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<polyline points="20 6 9 17 4 12" />
								</svg>
							</button>
							<button
								class="icon-action"
								title="Unstage"
								onclick={(e: MouseEvent) => { e.stopPropagation(); handleUnstageFile(file.path); }}
							>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M18 6L6 18M6 6l12 12" />
								</svg>
							</button>
							<button
								class="icon-action danger"
								title="Revert"
								onclick={(e: MouseEvent) => { e.stopPropagation(); handleRevertFile(file.path); }}
							>
								<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
									<path d="M3 3v5h5" />
								</svg>
							</button>
						</span>
					</div>
				{/each}
			</div>
		</div>
	</div>
{/if}

<style>
	.git-tab {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--text-secondary);
		font-size: 13px;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.branch-select-wrapper {
		flex: 1;
	}

	.branch-select {
		width: 100%;
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		color: var(--text-primary);
		font-size: 12px;
		padding: 5px 8px;
		border-radius: var(--radius-button);
		font-family: var(--font-code);
		cursor: pointer;
	}

	.branch-select:focus {
		border-color: var(--accent);
	}

	.branch-select option {
		background: var(--bg-primary);
		color: var(--text-primary);
	}

	.refresh-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		padding: 5px;
		border-radius: var(--radius-button);
		cursor: pointer;
		transition: color 0.15s, background 0.15s;
	}

	.refresh-btn:hover {
		color: var(--text-primary);
		background: var(--bg-surface);
	}

	.refresh-btn:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.changes-section {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.section-title {
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		letter-spacing: 0.5px;
	}

	.section-actions {
		display: flex;
		gap: 6px;
	}

	.action-btn {
		background: none;
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		font-size: 11px;
		padding: 2px 8px;
		border-radius: var(--radius-button);
		cursor: pointer;
		font-family: inherit;
		transition: color 0.15s, background 0.15s;
	}

	.action-btn:hover {
		color: var(--text-primary);
		background: var(--bg-surface);
	}

	.empty-changes {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		color: var(--text-secondary);
		font-size: 12px;
	}

	.file-list {
		flex: 1;
		overflow-y: auto;
	}

	.file-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 12px;
		background: none;
		border: none;
		border-bottom: 1px solid var(--border-color);
		color: var(--text-primary);
		font-size: 12px;
		cursor: pointer;
		font-family: inherit;
		text-align: left;
		transition: background 0.1s;
	}

	.file-item:hover {
		background: var(--bg-surface);
	}

	.file-status {
		font-family: var(--font-code);
		font-size: 11px;
		font-weight: 700;
		width: 16px;
		text-align: center;
		flex-shrink: 0;
	}

	.status-modified {
		color: #fbbf24;
	}

	.status-added {
		color: #4ade80;
	}

	.status-deleted {
		color: #f87171;
	}

	.status-untracked {
		color: var(--text-secondary);
	}

	.file-path {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-family: var(--font-code);
		font-size: 12px;
	}

	.file-stats {
		display: flex;
		gap: 6px;
		font-family: var(--font-code);
		font-size: 11px;
		flex-shrink: 0;
	}

	.stat-add {
		color: #4ade80;
	}

	.stat-del {
		color: #f87171;
	}

	.file-actions {
		display: flex;
		gap: 4px;
		opacity: 0;
		transition: opacity 0.15s;
		flex-shrink: 0;
	}

	.file-item:hover .file-actions {
		opacity: 1;
	}

	.icon-action {
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		padding: 3px;
		border-radius: 3px;
		cursor: pointer;
		transition: color 0.15s, background 0.15s;
	}

	.icon-action:hover {
		color: var(--text-primary);
		background: var(--bg-surface);
	}

	.icon-action.danger:hover {
		color: #f87171;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		background: rgba(239, 68, 68, 0.15);
		border-bottom: 1px solid rgba(239, 68, 68, 0.3);
		color: #f87171;
		font-size: 12px;
		flex-shrink: 0;
	}

	.error-dismiss {
		background: none;
		border: none;
		color: #f87171;
		font-size: 16px;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
	}
</style>
