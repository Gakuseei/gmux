<script lang="ts">
	import { onMount } from 'svelte';
	import { appStore } from '$lib/stores/app.svelte';
	import { persistence } from '$lib/stores/persistence.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import TopBar from '$lib/components/TopBar.svelte';
	import Sidebar from '$lib/components/sidebar/Sidebar.svelte';
	import StatusBar from '$lib/components/statusbar/StatusBar.svelte';
	import TerminalView from '$lib/components/terminal/TerminalView.svelte';
	import InsightsView from '$lib/components/insights/InsightsView.svelte';
	import NewWorkspaceModal from '$lib/components/workspace/NewWorkspaceModal.svelte';
	import SettingsModal from '$lib/components/settings/SettingsModal.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { initKeybindings } from '$lib/utils/keybindings';

	let loaded = $state(false);

	onMount(() => {
		settingsStore.load().then(() => {
			settingsStore.applyAppearance();
		});

		persistence.loadState().then(async () => {
			loaded = true;
			if (persistence.windowState) {
				const win = getCurrentWindow();
				const ws = persistence.windowState;
				await win.setSize(new (await import('@tauri-apps/api/dpi')).PhysicalSize(ws.width, ws.height));
				await win.setPosition(new (await import('@tauri-apps/api/dpi')).PhysicalPosition(ws.x, ws.y));
			}
		});

		let unlistenClose: (() => void) | null = null;

		getCurrentWindow().onCloseRequested(async () => {
			const win = getCurrentWindow();
			const size = await win.innerSize();
			const pos = await win.innerPosition();
			await persistence.saveState({
				width: size.width,
				height: size.height,
				x: pos.x,
				y: pos.y,
			});
		}).then((fn) => {
			unlistenClose = fn;
		});

		const cleanupKeybindings = initKeybindings();

		return () => {
			if (unlistenClose) unlistenClose();
			cleanupKeybindings();
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

{#if appStore.showSettings}
	<SettingsModal />
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
