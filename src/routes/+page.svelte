<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import StatusBar from '$lib/components/statusbar/StatusBar.svelte';
	import TerminalPane from '$lib/components/terminal/TerminalPane.svelte';
</script>

<div class="app-shell" style:grid-template-columns="{appStore.sidebarMinimized ? '48px' : appStore.sidebarWidth + 'px'} 1fr">
	<div class="topbar-area">
		<TopBar />
	</div>

	<div class="sidebar-area"></div>

	<div class="content-area">
		{#if appStore.activeView === 'terminals'}
			<TerminalPane terminalId="proof-of-concept" shell="/bin/fish" cwd={"/home/eriks"} />
		{:else}
			<div class="insights-placeholder">Insights</div>
		{/if}
	</div>

	<div class="statusbar-area">
		<StatusBar />
	</div>
</div>

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

	.insights-placeholder {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--text-secondary);
		font-size: 14px;
	}
</style>
