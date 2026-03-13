<script lang="ts">
	import type { TerminalSession } from '$lib/types/workspace';

	let {
		session,
		onSplitHorizontal,
		onSplitVertical,
		onClose
	}: {
		session: TerminalSession;
		onSplitHorizontal: () => void;
		onSplitVertical: () => void;
		onClose: () => void;
	} = $props();
</script>

<div class="pane-header">
	<div class="left">
		<span class="name">{session.name}</span>
		{#if session.bypassPermissions}
			<span class="bypass-badge">&#9889;bypass</span>
		{/if}
	</div>
	<div class="right">
		<button class="header-btn" onclick={onSplitHorizontal} title="Split horizontal">&#8862;</button>
		<button class="header-btn" onclick={onSplitVertical} title="Split vertical">&#8863;</button>
		<button class="header-btn close-btn" onclick={onClose} title="Close">&#10005;</button>
	</div>
</div>

<style>
	.pane-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 24px;
		padding: 0 8px;
		background: var(--bg-surface);
		border-bottom: 1px solid var(--border-color);
		font-size: 12px;
		flex-shrink: 0;
		user-select: none;
	}

	.left {
		display: flex;
		align-items: center;
		gap: 6px;
		overflow: hidden;
	}

	.name {
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.bypass-badge {
		color: var(--accent);
		font-size: 10px;
		white-space: nowrap;
	}

	.right {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.header-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		padding: 2px 4px;
		font-size: 12px;
		line-height: 1;
		border-radius: 3px;
	}

	.header-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.08);
	}

	.close-btn:hover {
		color: #ef4444;
		background: rgba(239, 68, 68, 0.15);
	}
</style>
