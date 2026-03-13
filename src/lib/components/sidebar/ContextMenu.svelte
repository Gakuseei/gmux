<script lang="ts">
	interface MenuItem {
		label: string;
		action: () => void;
		danger?: boolean;
	}

	let { items, x, y, onClose }: { items: MenuItem[]; x: number; y: number; onClose: () => void } =
		$props();

	let menuEl: HTMLDivElement | undefined = $state();
	let focusedIndex = $state(0);

	function focusItem(index: number) {
		if (!menuEl) return;
		const buttons = menuEl.querySelectorAll<HTMLButtonElement>('[role="menuitem"]');
		if (buttons[index]) {
			focusedIndex = index;
			buttons[index].focus();
		}
	}

	function handleMenuKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onClose();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			focusItem((focusedIndex + 1) % items.length);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			focusItem((focusedIndex - 1 + items.length) % items.length);
		} else if (e.key === 'Home') {
			e.preventDefault();
			focusItem(0);
		} else if (e.key === 'End') {
			e.preventDefault();
			focusItem(items.length - 1);
		}
	}

	function handleWindowClick() {
		onClose();
	}

	$effect(() => {
		const timer = setTimeout(() => {
			window.addEventListener('click', handleWindowClick);
			focusItem(0);
		}, 0);
		return () => {
			clearTimeout(timer);
			window.removeEventListener('click', handleWindowClick);
		};
	});
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
	bind:this={menuEl}
	class="context-menu"
	style:left="{x}px"
	style:top="{y}px"
	role="menu"
	onkeydown={handleMenuKeydown}
>
	{#each items as item, i}
		<button
			class="menu-item"
			class:danger={item.danger}
			role="menuitem"
			tabindex={i === focusedIndex ? 0 : -1}
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
