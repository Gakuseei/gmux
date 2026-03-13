<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
	import { splitNode, removeNode } from '$lib/utils/layout-helpers';
	import type { TerminalSession } from '$lib/types/workspace';
	import SplitContainer from './SplitContainer.svelte';

	function handleSplit(terminalId: string, direction: 'horizontal' | 'vertical') {
		const ws = appStore.activeWorkspace;
		if (!ws) return;

		const newSession: TerminalSession = {
			id: crypto.randomUUID(),
			name: `Terminal ${ws.sessions.length + 1}`,
			shell: '/bin/fish',
			cwd: ws.cwd,
			status: 'running',
			notificationCount: 0
		};

		ws.sessions.push(newSession);
		ws.layout = splitNode(ws.layout, terminalId, direction, newSession);
	}

	function handleClose(terminalId: string) {
		const ws = appStore.activeWorkspace;
		if (!ws) return;

		const newLayout = removeNode(ws.layout, terminalId);
		if (newLayout) {
			ws.layout = newLayout;
			ws.sessions = ws.sessions.filter((s) => s.id !== terminalId);
		}
	}

	function addTerminal() {
		const ws = appStore.activeWorkspace;
		if (!ws) return;

		const firstTerminalId = findFirstTerminalId(ws.layout);
		if (firstTerminalId) {
			handleSplit(firstTerminalId, 'horizontal');
		}
	}

	function findFirstTerminalId(node: import('$lib/types/workspace').SplitNode): string | null {
		if (node.type === 'terminal') return node.terminalId ?? null;
		if (node.children) {
			for (const child of node.children) {
				const id = findFirstTerminalId(child);
				if (id) return id;
			}
		}
		return null;
	}
</script>

<div class="terminal-view">
	{#if appStore.activeWorkspace}
		<div class="split-area">
			<SplitContainer
				node={appStore.activeWorkspace.layout}
				workspace={appStore.activeWorkspace}
				onSplit={handleSplit}
				onClose={handleClose}
			/>
		</div>
		<div class="bottom-bar">
			<button class="bar-btn" onclick={() => (appStore.showNewWorkspaceModal = true)}>
				New Workspace
			</button>
			<button class="bar-btn" onclick={addTerminal}>
				+ Terminal
			</button>
		</div>
	{:else}
		<div class="empty-state">
			<p>Create a workspace to get started</p>
			<button class="create-btn" onclick={() => (appStore.showNewWorkspaceModal = true)}>
				New Workspace
			</button>
		</div>
	{/if}
</div>

<style>
	.terminal-view {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.split-area {
		flex: 1 1 auto;
		display: flex;
		overflow: hidden;
		min-height: 0;
	}

	.bottom-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 8px;
		background: var(--bg-surface);
		border-top: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.bar-btn {
		background: none;
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		padding: 3px 10px;
		font-size: 12px;
		border-radius: 4px;
		cursor: pointer;
	}

	.bar-btn:hover {
		color: var(--text-primary);
		border-color: var(--accent);
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
		color: var(--text-secondary);
		font-size: 14px;
	}

	.create-btn {
		background: var(--accent);
		border: none;
		color: #fff;
		padding: 8px 20px;
		font-size: 13px;
		border-radius: 6px;
		cursor: pointer;
	}

	.create-btn:hover {
		opacity: 0.9;
	}
</style>
