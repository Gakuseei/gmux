<script lang="ts">
	import type { SplitNode, Workspace } from '$lib/types/workspace';
	import PaneHeader from './PaneHeader.svelte';
	import TerminalPane from './TerminalPane.svelte';
	import SplitContainer from './SplitContainer.svelte';

	let {
		node,
		workspace,
		onSplit,
		onClose,
		onTerminalData,
		onRatioChange
	}: {
		node: SplitNode;
		workspace: Workspace;
		onSplit: (terminalId: string, direction: 'horizontal' | 'vertical') => void;
		onClose: (terminalId: string) => void;
		onTerminalData?: (terminalId: string, data: string) => void;
		onRatioChange?: (node: SplitNode, newRatio: number) => void;
	} = $props();

	let initialRatio = $derived(node.ratio ?? 0.5);
	let ratioOffset = $state(0);
	let ratio = $derived(initialRatio + ratioOffset);
	let dragging = $state(false);
	let containerEl: HTMLDivElement | undefined = $state();

	function findSession(terminalId: string) {
		return workspace.sessions.find((s) => s.id === terminalId);
	}

	function onDividerMouseDown(e: MouseEvent) {
		e.preventDefault();
		dragging = true;

		const startX = e.clientX;
		const startY = e.clientY;
		const startOffset = ratioOffset;
		const rect = containerEl?.getBoundingClientRect();
		if (!rect) return;

		const isHorizontal = node.direction === 'horizontal';

		function onMouseMove(e: MouseEvent) {
			if (!rect) return;
			const delta = isHorizontal ? e.clientX - startX : e.clientY - startY;
			const totalSize = isHorizontal ? rect.width : rect.height;
			const rawRatio = initialRatio + startOffset + delta / totalSize;
			ratioOffset = Math.min(0.9, Math.max(0.1, rawRatio)) - initialRatio;
		}

		function onMouseUp() {
			dragging = false;
			window.removeEventListener('mousemove', onMouseMove);
			window.removeEventListener('mouseup', onMouseUp);
			const finalRatio = Math.min(0.9, Math.max(0.1, initialRatio + ratioOffset));
			node.ratio = finalRatio;
			if (onRatioChange) {
				onRatioChange(node, finalRatio);
			}
		}

		window.addEventListener('mousemove', onMouseMove);
		window.addEventListener('mouseup', onMouseUp);
	}
</script>

{#if node.type === 'terminal'}
	{@const session = findSession(node.terminalId ?? '')}
	{#if session}
		<div class="terminal-pane">
			<PaneHeader
				{session}
				onSplitHorizontal={() => onSplit(node.terminalId!, 'horizontal')}
				onSplitVertical={() => onSplit(node.terminalId!, 'vertical')}
				onClose={() => onClose(node.terminalId!)}
			/>
			<div class="terminal-content">
				<TerminalPane
					terminalId={session.id}
					shell={session.shell}
					cwd={session.cwd}
					command={session.command}
					existingPtyId={session.ptyId}
					onData={onTerminalData ? (data) => onTerminalData!(session.id, data) : undefined}
				/>
			</div>
		</div>
	{/if}
{:else if node.children && node.children.length === 2}
	<div
		bind:this={containerEl}
		class="split-container"
		class:horizontal={node.direction === 'horizontal'}
		class:vertical={node.direction === 'vertical'}
		class:dragging
	>
		<div class="split-child" style:flex-basis="{ratio * 100}%">
			<SplitContainer
				node={node.children[0]}
				{workspace}
				{onSplit}
				{onClose}
				{onTerminalData}
				{onRatioChange}
			/>
		</div>
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="divider"
			class:divider-horizontal={node.direction === 'horizontal'}
			class:divider-vertical={node.direction === 'vertical'}
			onmousedown={onDividerMouseDown}
			role="separator"
			tabindex="-1"
		></div>
		<div class="split-child" style:flex-basis="{(1 - ratio) * 100}%">
			<SplitContainer
				node={node.children[1]}
				{workspace}
				{onSplit}
				{onClose}
				{onTerminalData}
				{onRatioChange}
			/>
		</div>
	</div>
{/if}

<style>
	.terminal-pane {
		display: flex;
		flex-direction: column;
		flex: 1 1 auto;
		overflow: hidden;
		min-width: 0;
		min-height: 0;
	}

	.terminal-content {
		flex: 1 1 auto;
		overflow: hidden;
		position: relative;
	}

	.split-container {
		display: flex;
		flex: 1 1 auto;
		overflow: hidden;
		min-width: 0;
		min-height: 0;
	}

	.split-container.horizontal {
		flex-direction: row;
	}

	.split-container.vertical {
		flex-direction: column;
	}

	.split-child {
		overflow: hidden;
		position: relative;
		display: flex;
		min-width: 0;
		min-height: 0;
	}

	.divider {
		flex-shrink: 0;
		background: var(--border-color);
		transition: background 0.15s;
		z-index: 1;
	}

	.divider:hover,
	.dragging .divider {
		background: var(--accent);
	}

	.divider-horizontal {
		width: 4px;
		cursor: col-resize;
	}

	.divider-vertical {
		height: 4px;
		cursor: row-resize;
	}
</style>
