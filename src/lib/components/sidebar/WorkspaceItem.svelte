<script lang="ts">
	import type { Workspace } from '$lib/types/workspace';
	import { appStore } from '$lib/stores/app.svelte';
	import ContextMenu from './ContextMenu.svelte';

	let {
		workspace,
		isActive,
		minimized
	}: { workspace: Workspace; isActive: boolean; minimized: boolean } = $props();

	let contextMenu = $state<{ x: number; y: number } | null>(null);
	let editing = $state(false);
	let editName = $state('');
	let editInput = $state<HTMLInputElement | null>(null);

	let abbreviation = $derived(
		workspace.name
			.split(/[\s-_]+/)
			.slice(0, 2)
			.map((w) => w[0]?.toUpperCase() ?? '')
			.join('') || workspace.name.slice(0, 2).toUpperCase()
	);

	let totalNotifications = $derived(
		workspace.sessions.reduce((sum, s) => sum + s.notificationCount, 0)
	);

	const STATUS_COLOR_MAP: Record<string, string> = {
		running: 'var(--color-success, #22c55e)',
		'needs-input': 'var(--notification, #3b82f6)',
		exited: '#6b7280'
	};

	const STATUS_COLOR_DEFAULT = '#6b7280';

	function statusColor(status: string): string {
		return STATUS_COLOR_MAP[status] ?? STATUS_COLOR_DEFAULT;
	}

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault();
		contextMenu = { x: e.clientX, y: e.clientY };
	}

	function startRename() {
		editName = workspace.name;
		editing = true;
		requestAnimationFrame(() => editInput?.select());
	}

	function commitRename() {
		const trimmed = editName.trim();
		if (trimmed && trimmed !== workspace.name) {
			appStore.renameWorkspace(workspace.id, trimmed);
		}
		editing = false;
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') commitRename();
		if (e.key === 'Escape') {
			editing = false;
		}
	}

	function handleDragStart(e: DragEvent) {
		if (e.dataTransfer) {
			e.dataTransfer.setData('text/plain', workspace.id);
			e.dataTransfer.effectAllowed = 'move';
		}
	}

	function handleItemKeydown(e: KeyboardEvent) {
		if ((e.key === 'F10' && e.shiftKey) || e.key === 'ContextMenu') {
			e.preventDefault();
			const target = e.currentTarget as HTMLElement;
			const rect = target.getBoundingClientRect();
			contextMenu = { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
		}
	}

	let contextMenuItems = $derived([
		{ label: 'Rename', action: startRename },
		...appStore.folders.map((f) => ({
			label: `Move to ${f.name}`,
			action: () => appStore.moveWorkspaceToFolder(workspace.id, f.id)
		})),
		...(workspace.folderId
			? [
					{
						label: 'Remove from folder',
						action: () => appStore.moveWorkspaceToFolder(workspace.id, undefined)
					}
				]
			: []),
		{ label: 'Close', action: () => appStore.removeWorkspace(workspace.id), danger: true }
	]);
</script>

{#if minimized}
	<button
		class="mini-item"
		class:active={isActive}
		onclick={() => appStore.setActiveWorkspace(workspace.id)}
		oncontextmenu={handleContextMenu}
		title={workspace.name}
	>
		<span class="abbreviation">{abbreviation}</span>
		{#if totalNotifications > 0}
			<span class="badge mini-badge">{totalNotifications}</span>
		{/if}
	</button>
{:else}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="workspace-item" class:active={isActive} oncontextmenu={handleContextMenu} onkeydown={handleItemKeydown} draggable="true" ondragstart={handleDragStart}>
		<button class="ws-header" onclick={() => appStore.setActiveWorkspace(workspace.id)}>
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
				<span class="ws-name">{workspace.name}</span>
			{/if}
		</button>

		{#if workspace.sessions.length > 0}
			<div class="sessions">
				{#each workspace.sessions as session}
					<div class="session-row">
						<span class="status-dot" style:background={statusColor(session.status)}></span>
						<span class="session-name">{session.name}</span>
						{#if session.notificationCount > 0}
							<span class="badge">{session.notificationCount}</span>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}

{#if contextMenu}
	<ContextMenu
		items={contextMenuItems}
		x={contextMenu.x}
		y={contextMenu.y}
		onClose={() => (contextMenu = null)}
	/>
{/if}

<style>
	.workspace-item {
		border-radius: var(--radius-button);
		margin: 1px 6px;
	}

	.workspace-item.active {
		background: rgba(255, 255, 255, 0.06);
	}

	.ws-header {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 5px 8px;
		background: none;
		border: none;
		color: var(--text-primary);
		font-size: 12px;
		font-family: inherit;
		cursor: pointer;
		text-align: left;
	}

	.ws-header:hover {
		background: rgba(255, 255, 255, 0.04);
		border-radius: var(--radius-button);
	}

	.ws-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.rename-input {
		background: var(--bg-surface);
		border: 1px solid var(--accent);
		color: var(--text-primary);
		font-size: 12px;
		font-family: inherit;
		padding: 1px 4px;
		border-radius: 3px;
		width: 100%;
	}

	.sessions {
		padding: 0 8px 4px 16px;
	}

	.session-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 0;
		font-size: 11px;
		color: var(--text-secondary);
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.session-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
	}

	.badge {
		background: var(--notification);
		color: #fff;
		font-size: 10px;
		font-weight: 600;
		padding: 0 5px;
		border-radius: 8px;
		min-width: 16px;
		text-align: center;
		line-height: 16px;
		flex-shrink: 0;
	}

	.mini-item {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		margin: 2px auto;
		background: none;
		border: none;
		border-radius: var(--radius-button);
		color: var(--text-secondary);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		position: relative;
	}

	.mini-item:hover,
	.mini-item.active {
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-primary);
	}

	.mini-badge {
		position: absolute;
		top: -2px;
		right: -2px;
		font-size: 9px;
		padding: 0 4px;
		min-width: 14px;
		line-height: 14px;
	}
</style>
