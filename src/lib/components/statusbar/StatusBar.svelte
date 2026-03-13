<footer class="statusbar">
	<span class="item">branch: &ndash;</span>
	<span class="sep">&middot;</span>
	<span class="item">model: &ndash;</span>
	<span class="sep">&middot;</span>
	<span class="item">reasoning: &ndash;</span>
	<span class="sep">&middot;</span>
	<span class="item">tokens: &ndash;</span>
	<span class="sep">&middot;</span>
	<span class="item">context: &ndash;</span>
</footer>

<style>
	.statusbar {
		display: flex;
		align-items: center;
		height: 28px;
		padding: 0 12px;
		background: var(--bg-surface);
		border-top: 1px solid var(--border-color);
		font-family: 'JetBrains Mono', monospace;
		font-size: 12px;
		color: var(--text-secondary);
		user-select: none;
		gap: 6px;
	}

	.sep {
		opacity: 0.5;
	}

	.item {
		white-space: nowrap;
	}
</style>
