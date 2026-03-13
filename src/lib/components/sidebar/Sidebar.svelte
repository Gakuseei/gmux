<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
	import FolderItem from './FolderItem.svelte';
	import WorkspaceItem from './WorkspaceItem.svelte';

	let resizing = $state(false);

	function startResize(e: MouseEvent) {
		e.preventDefault();
		resizing = true;
		const onMouseMove = (ev: MouseEvent) => {
			const width = Math.min(400, Math.max(180, ev.clientX));
			appStore.sidebarWidth = width;
		};
		const onMouseUp = () => {
			resizing = false;
			window.removeEventListener('mousemove', onMouseMove);
			window.removeEventListener('mouseup', onMouseUp);
		};
		window.addEventListener('mousemove', onMouseMove);
		window.addEventListener('mouseup', onMouseUp);
	}
</script>

<aside class="sidebar" class:minimized={appStore.sidebarMinimized} class:resizing>
	{#if appStore.sidebarMinimized}
		<div class="mini-content">
			<button class="mini-action" title="New Workspace" onclick={() => (appStore.showNewWorkspaceModal = true)}>
				+
			</button>

			<div class="mini-list">
				{#each appStore.workspaces as ws (ws.id)}
					<WorkspaceItem
						workspace={ws}
						isActive={appStore.activeWorkspaceId === ws.id}
						minimized={true}
					/>
				{/each}
			</div>

			<button class="toggle-btn" onclick={() => appStore.toggleSidebar()} title="Expand sidebar">
				&#9654;
			</button>
		</div>
	{:else}
		<div class="expanded-content">
			<button class="new-workspace-btn" onclick={() => (appStore.showNewWorkspaceModal = true)}>
				+ New Workspace
			</button>

			<div class="section-label">WORKSPACES</div>

			<div class="workspace-list">
				{#each appStore.folders as folder (folder.id)}
					<FolderItem
						{folder}
						workspaces={appStore.workspacesByFolder.get(folder.id) ?? []}
					/>
				{/each}

				{#each appStore.ungroupedWorkspaces as ws (ws.id)}
					<WorkspaceItem
						workspace={ws}
						isActive={appStore.activeWorkspaceId === ws.id}
						minimized={false}
					/>
				{/each}
			</div>

			<div class="sidebar-footer">
				<button class="footer-btn" onclick={() => appStore.addFolder('New Folder')}>
					New Folder
				</button>
				<button class="toggle-btn" onclick={() => appStore.toggleSidebar()} title="Minimize sidebar">
					&#9664;
				</button>
			</div>
		</div>

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="resize-handle" onmousedown={startResize}></div>
	{/if}
</aside>

<style>
	.sidebar {
		position: relative;
		height: 100%;
		display: flex;
		flex-direction: column;
		user-select: none;
		overflow: hidden;
	}

	.sidebar.resizing {
		cursor: col-resize;
	}

	.expanded-content {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.new-workspace-btn {
		display: block;
		width: calc(100% - 16px);
		margin: 8px 8px 4px;
		padding: 6px 0;
		background: rgba(255, 255, 255, 0.06);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		font-size: 12px;
		font-family: inherit;
		cursor: pointer;
		transition: background 0.15s;
	}

	.new-workspace-btn:hover {
		background: rgba(255, 255, 255, 0.1);
	}

	.section-label {
		padding: 10px 14px 4px;
		font-size: 10px;
		font-weight: 600;
		color: var(--text-secondary);
		letter-spacing: 0.5px;
	}

	.workspace-list {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.workspace-list::-webkit-scrollbar {
		width: 4px;
	}

	.workspace-list::-webkit-scrollbar-thumb {
		background: rgba(255, 255, 255, 0.1);
		border-radius: 2px;
	}

	.sidebar-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 8px;
		border-top: 1px solid var(--border-color);
	}

	.footer-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 11px;
		font-family: inherit;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: var(--radius-button);
	}

	.footer-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.06);
	}

	.toggle-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 10px;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: var(--radius-button);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.toggle-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.06);
	}

	.resize-handle {
		position: absolute;
		top: 0;
		right: 0;
		width: 4px;
		height: 100%;
		cursor: col-resize;
	}

	.resize-handle:hover {
		background: var(--accent);
	}

	.mini-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		height: 100%;
		padding-top: 8px;
	}

	.mini-action {
		width: 32px;
		height: 32px;
		background: none;
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		font-size: 16px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 8px;
	}

	.mini-action:hover {
		background: rgba(255, 255, 255, 0.06);
	}

	.mini-list {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		width: 100%;
	}

	.mini-list::-webkit-scrollbar {
		width: 0;
	}

	.minimized .toggle-btn {
		margin-top: auto;
		margin-bottom: 6px;
	}
</style>
