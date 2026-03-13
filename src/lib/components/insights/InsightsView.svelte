<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
	import { gitStore } from '$lib/stores/git.svelte';
	import type { InsightsTab } from '$lib/types/workspace';
	import UsageTab from './UsageTab.svelte';
	import GitTab from './GitTab.svelte';
	import InfoTab from './InfoTab.svelte';

	const tabs: { id: InsightsTab; label: string }[] = [
		{ id: 'usage', label: 'Usage' },
		{ id: 'git', label: 'Git' },
		{ id: 'info', label: 'Info' },
	];

	$effect(() => {
		if (appStore.activeInsightsTab === 'git' && appStore.activeWorkspace?.cwd) {
			gitStore.setRepoPath(appStore.activeWorkspace.cwd);
		}
	});
</script>

<div class="insights-layout">
	<nav class="insights-nav">
		{#each tabs as tab (tab.id)}
			<button
				class="nav-item"
				class:active={appStore.activeInsightsTab === tab.id}
				onclick={() => (appStore.activeInsightsTab = tab.id)}
			>
				{tab.label}
			</button>
		{/each}
	</nav>

	<div class="insights-content">
		{#if appStore.activeInsightsTab === 'usage'}
			<UsageTab />
		{:else if appStore.activeInsightsTab === 'git'}
			<GitTab />
		{:else if appStore.activeInsightsTab === 'info'}
			<InfoTab />
		{/if}
	</div>
</div>

<style>
	.insights-layout {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.insights-nav {
		width: 180px;
		min-width: 180px;
		display: flex;
		flex-direction: column;
		padding: 12px 8px;
		gap: 2px;
		border-right: 1px solid var(--border-color);
		background: var(--bg-sidebar);
	}

	.nav-item {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 13px;
		font-family: inherit;
		padding: 8px 12px;
		border-radius: var(--radius-button);
		cursor: pointer;
		text-align: left;
		transition: color 0.15s, background 0.15s;
	}

	.nav-item:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.04);
	}

	.nav-item.active {
		color: var(--text-primary);
		background: rgba(16, 163, 127, 0.15);
		border-left: 2px solid var(--accent);
		padding-left: 10px;
	}

	.insights-content {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.insights-content::-webkit-scrollbar {
		width: 6px;
	}

	.insights-content::-webkit-scrollbar-thumb {
		background: rgba(255, 255, 255, 0.1);
		border-radius: 3px;
	}

</style>
