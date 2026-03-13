<script lang="ts">
	interface MenuItem {
		label: string;
		action: () => void;
		danger?: boolean;
	}

	let { items, x, y, onClose }: { items: MenuItem[]; x: number; y: number; onClose: () => void } =
		$props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	function handleWindowClick() {
		onClose();
	}

	$effect(() => {
		window.addEventListener('keydown', handleKeydown);
		const timer = setTimeout(() => window.addEventListener('click', handleWindowClick), 0);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
			clearTimeout(timer);
			window.removeEventListener('click', handleWindowClick);
		};
	});
</script>

<div class="context-menu" style:left="{x}px" style:top="{y}px">
	{#each items as item}
		<button
			class="menu-item"
			class:danger={item.danger}
			onclick={(e) => {
				e.stopPropagation();
				item.action();
				onClose();
			}}
		>
			{item.label}
		</button>
	{/each}
</div>

<style>
	.context-menu {
		position: fixed;
		z-index: 1000;
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		padding: 4px 0;
		min-width: 140px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
	}

	.menu-item {
		display: block;
		width: 100%;
		text-align: left;
		background: none;
		border: none;
		color: var(--text-primary);
		font-size: 12px;
		font-family: inherit;
		padding: 6px 12px;
		cursor: pointer;
	}

	.menu-item:hover {
		background: rgba(255, 255, 255, 0.06);
	}

	.menu-item.danger {
		color: #ef4444;
	}

	.menu-item.danger:hover {
		background: rgba(239, 68, 68, 0.1);
	}
</style>
