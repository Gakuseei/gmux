<script lang="ts">
	import { statusStore } from '$lib/stores/status.svelte';
	import { formatTokens } from '$lib/utils/format';
</script>

<footer class="statusbar">
	<span class="item">&#9095; {statusStore.gitBranch}</span>
	<span class="sep">&#183;</span>
	<span class="item">{statusStore.model}</span>
	<span class="sep">&#183;</span>
	<span class="item">reasoning: {statusStore.reasoning}</span>
	<span class="sep">&#183;</span>
	<span class="item">&#8593;{formatTokens(statusStore.inputTokens)} &#8595;{formatTokens(statusStore.outputTokens)}</span>
	<span class="sep">&#183;</span>
	<span class="item">ctx {statusStore.contextPercent}% ({formatTokens(statusStore.contextUsed)}/{formatTokens(statusStore.contextTotal)})</span>
</footer>

<style>
	.statusbar {
		display: flex;
		align-items: center;
		height: 28px;
		padding: 0 12px;
		background: var(--bg-surface);
		border-top: 1px solid var(--border-color);
		font-family: var(--font-code);
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
