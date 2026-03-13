<script lang="ts">
	import type { FileDiff } from '$lib/stores/git.svelte';

	let { diff, onBack }: { diff: FileDiff; onBack: () => void } = $props();

	let totalAdditions = $derived(
		diff.hunks.reduce(
			(sum, hunk) => sum + hunk.filter((l) => l.origin === '+').length,
			0
		)
	);

	let totalDeletions = $derived(
		diff.hunks.reduce(
			(sum, hunk) => sum + hunk.filter((l) => l.origin === '-').length,
			0
		)
	);
</script>

<div class="diff-view">
	<div class="diff-header">
		<button class="back-btn" onclick={onBack}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M19 12H5M12 19l-7-7 7-7" />
			</svg>
			Back
		</button>
		<span class="diff-filename">{diff.path}</span>
		<span class="diff-stats">
			<span class="stat-add">+{totalAdditions}</span>
			<span class="stat-del">-{totalDeletions}</span>
		</span>
	</div>

	<div class="diff-content">
		{#each diff.hunks as hunk, i}
			{#if i > 0}
				<div class="hunk-separator">...</div>
			{/if}
			{#each hunk as line}
				<div
					class="diff-line"
					class:line-add={line.origin === '+'}
					class:line-del={line.origin === '-'}
				>
					<span class="line-num old">{line.oldLineno ?? ''}</span>
					<span class="line-num new">{line.newLineno ?? ''}</span>
					<span class="line-origin">{line.origin}</span>
					<span class="line-content">{line.content}</span>
				</div>
			{/each}
		{/each}
	</div>
</div>

<style>
	.diff-view {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.diff-header {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.back-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		background: none;
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		font-size: 12px;
		padding: 4px 8px;
		border-radius: var(--radius-button);
		cursor: pointer;
		font-family: inherit;
		transition: color 0.15s, background 0.15s;
	}

	.back-btn:hover {
		color: var(--text-primary);
		background: var(--bg-surface);
	}

	.diff-filename {
		font-size: 12px;
		font-family: var(--font-code);
		color: var(--text-primary);
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.diff-stats {
		display: flex;
		gap: 8px;
		font-size: 12px;
		font-family: var(--font-code);
	}

	.stat-add {
		color: #4ade80;
	}

	.stat-del {
		color: #f87171;
	}

	.diff-content {
		flex: 1;
		overflow: auto;
		font-family: var(--font-code);
		font-size: 12px;
		line-height: 1.5;
	}

	.diff-line {
		display: flex;
		white-space: pre;
		min-width: fit-content;
	}

	.line-add {
		background: var(--diff-add);
	}

	.line-del {
		background: var(--diff-delete);
	}

	.line-num {
		display: inline-block;
		width: 48px;
		text-align: right;
		padding-right: 8px;
		color: var(--text-secondary);
		opacity: 0.5;
		user-select: none;
		flex-shrink: 0;
	}

	.line-origin {
		display: inline-block;
		width: 16px;
		text-align: center;
		color: var(--text-secondary);
		user-select: none;
		flex-shrink: 0;
	}

	.line-add .line-origin {
		color: #4ade80;
	}

	.line-del .line-origin {
		color: #f87171;
	}

	.line-content {
		flex: 1;
		padding-right: 16px;
	}

	.hunk-separator {
		padding: 4px 0;
		text-align: center;
		color: var(--text-secondary);
		font-size: 11px;
		background: var(--bg-surface);
		border-top: 1px solid var(--border-color);
		border-bottom: 1px solid var(--border-color);
	}
</style>
