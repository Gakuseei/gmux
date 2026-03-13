<script lang="ts">
	import type { Folder, Workspace } from '$lib/types/workspace';
	import { appStore } from '$lib/stores/app.svelte';
	import WorkspaceItem from './WorkspaceItem.svelte';
	import ContextMenu from './ContextMenu.svelte';

	let { folder, workspaces }: { folder: Folder; workspaces: Workspace[] } = $props();

	let contextMenu = $state<{ x: number; y: number } | null>(null);
	let editing = $state(false);
	let editName = $state('');
	let editInput = $state<HTMLInputElement | null>(null);
	let dragOver = $state(false);

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = 'move';
		}
		dragOver = true;
	}

	function handleDragLeave() {
		dragOver = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;
		const workspaceId = e.dataTransfer?.getData('text/plain');
		if (workspaceId) {
			appStore.moveWorkspaceToFolder(workspaceId, folder.id);
		}
	}

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault();
		contextMenu = { x: e.clientX, y: e.clientY };
	}

	function startRename() {
		editName = folder.name;
		editing = true;
		requestAnimationFrame(() => editInput?.select());
	}

	function commitRename() {
		const trimmed = editName.trim();
		if (trimmed && trimmed !== folder.name) {
			appStore.renameFolder(folder.id, trimmed);
		}
		editing = false;
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') commitRename();
		if (e.key === 'Escape') {
			editing = false;
		}
	}

	let contextMenuItems = $derived([
		{ label: 'Rename', action: startRename },
		{ label: 'Delete', action: () => appStore.removeFolder(folder.id), danger: true }
	]);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="folder" class:drag-over={dragOver} ondragover={handleDragOver} ondragleave={handleDragLeave} ondrop={handleDrop} role="group">
	<button class="folder-header" onclick={() => appStore.toggleFolder(folder.id)} oncontextmenu={handleContextMenu}>
		<span class="arrow">{folder.collapsed ? '\u25B6' : '\u25BC'}</span>
		{#if editing}
			<input
				bind:this={editInput}
				bind:value={editName}
				class="rename-input"
				onblur={commitRename}
				onkeydown={handleEditKeydown}
				onclick={(e) => e.stopPropagation()}
			/>
		{:else}
			<span class="folder-name">{folder.name}</span>
		{/if}
	</button>

	{#if !folder.collapsed}
		<div class="folder-children">
			{#each workspaces as ws (ws.id)}
				<WorkspaceItem
					workspace={ws}
					isActive={appStore.activeWorkspaceId === ws.id}
					minimized={false}
				/>
			{/each}
		</div>
	{/if}
</div>

{#if contextMenu}
	<ContextMenu
		items={contextMenuItems}
		x={contextMenu.x}
		y={contextMenu.y}
		onClose={() => (contextMenu = null)}
	/>
{/if}

<style>
	.folder {
		margin: 2px 0;
		border-radius: var(--radius-button);
		transition: background 0.15s;
	}

	.folder.drag-over {
		background: rgba(16, 163, 127, 0.1);
		outline: 1px dashed var(--accent);
	}

	.folder-header {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 4px 12px;
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 11px;
		font-family: inherit;
		cursor: pointer;
		text-transform: uppercase;
		letter-spacing: 0.3px;
		font-weight: 600;
	}

	.folder-header:hover {
		color: var(--text-primary);
	}

	.arrow {
		font-size: 8px;
		width: 10px;
		flex-shrink: 0;
	}

	.folder-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.rename-input {
		background: var(--bg-surface);
		border: 1px solid var(--accent);
		color: var(--text-primary);
		font-size: 11px;
		font-family: inherit;
		padding: 1px 4px;
		border-radius: 3px;
		flex: 1;
		text-transform: none;
	}

	.folder-children {
		padding-left: 4px;
	}
</style>
