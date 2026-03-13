<script lang="ts">
	import { onMount } from 'svelte';
	import { appStore } from '$lib/stores/app.svelte';
	import { persistence } from '$lib/stores/persistence.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
	import StatusBar from '$lib/components/statusbar/StatusBar.svelte';
	import TerminalView from '$lib/components/terminal/TerminalView.svelte';
	import InsightsView from '$lib/components/insights/InsightsView.svelte';
	import NewWorkspaceModal from '$lib/components/workspace/NewWorkspaceModal.svelte';

	let loaded = $state(false);

	onMount(() => {
		persistence.loadState().then(() => {
			loaded = true;
		});

		const handleBeforeUnload = () => {
			persistence.saveState();
		};
		window.addEventListener('beforeunload', handleBeforeUnload);

		return () => {
			window.removeEventListener('beforeunload', handleBeforeUnload);
		};
	});

	$effect(() => {
		if (!loaded) return;
		JSON.stringify(appStore.workspaces);
		JSON.stringify(appStore.folders);
		appStore.sidebarWidth;
		appStore.sidebarMinimized;
		appStore.activeWorkspaceId;
		persistence.scheduleSave();
	});
</script>

<div class="app-shell" style:grid-template-columns="{appStore.sidebarMinimized ? '48px' : appStore.sidebarWidth + 'px'} 1fr">
	<div class="topbar-area">
		<TopBar />
	</div>

	<div class="sidebar-area">
		<Sidebar />
	</div>

	<div class="content-area">
		{#if appStore.activeView === 'terminals'}
			<TerminalView />
		{:else if appStore.activeView === 'insights'}
			<InsightsView />
		{/if}
	</div>

	<div class="statusbar-area">
		<StatusBar />
	</div>
</div>

{#if appStore.showNewWorkspaceModal}
	<NewWorkspaceModal />
{/if}

<style>
	.app-shell {
		display: grid;
		grid-template-rows: 40px 1fr 28px;
		height: 100vh;
		overflow: hidden;
	}

	.topbar-area {
		grid-column: 1 / -1;
	}

	.sidebar-area {
		background: var(--bg-sidebar);
		border-right: 1px solid var(--border-color);
		overflow: hidden;
	}

	.content-area {
		overflow: hidden;
		background: var(--bg-primary);
	}

	.statusbar-area {
		grid-column: 1 / -1;
	}

</style>
