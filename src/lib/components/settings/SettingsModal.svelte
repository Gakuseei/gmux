<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
	import { createFocusTrap } from '$lib/utils/focus-trap';
	import AppearanceSettings from './AppearanceSettings.svelte';
	import TerminalSettings from './TerminalSettings.svelte';
	import AiCliSettings from './AiCliSettings.svelte';
	import RateLimitSettings from './RateLimitSettings.svelte';
	import NotificationSettings from './NotificationSettings.svelte';
	import KeybindingSettings from './KeybindingSettings.svelte';

	type Section = 'appearance' | 'terminal' | 'ai-clis' | 'rate-limits' | 'notifications' | 'keybindings';

	const NAV_ITEMS: { id: Section; label: string }[] = [
		{ id: 'appearance', label: 'Appearance' },
		{ id: 'terminal', label: 'Terminal' },
		{ id: 'ai-clis', label: 'AI CLIs' },
		{ id: 'rate-limits', label: 'Rate Limits' },
		{ id: 'notifications', label: 'Notifications' },
		{ id: 'keybindings', label: 'Keybindings' },
	];

	let activeSection = $state<Section>('appearance');

	function close() {
		appStore.showSettings = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && activeSection !== 'keybindings') close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) close();
	}

	let modalEl: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (!modalEl) return;
		const cleanup = createFocusTrap(modalEl);
		return cleanup;
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={handleBackdropClick} onkeydown={handleKeydown}>
	<!-- svelte-ignore a11y_interactive_supports_focus -->
	<div bind:this={modalEl} class="modal" role="dialog" aria-modal="true" aria-labelledby="settings-modal-title" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
		<div class="modal-header">
			<h2 id="settings-modal-title">Settings</h2>
			<button class="close-btn" aria-label="Close modal" onclick={close}>&times;</button>
		</div>

		<div class="modal-content">
			<nav class="sidebar-nav">
				{#each NAV_ITEMS as item}
					<button
						class="nav-item"
						class:active={activeSection === item.id}
						onclick={() => (activeSection = item.id)}
					>
						{item.label}
					</button>
				{/each}
			</nav>

			<div class="section-content">
				{#if activeSection === 'appearance'}
					<AppearanceSettings />
				{:else if activeSection === 'terminal'}
					<TerminalSettings />
				{:else if activeSection === 'ai-clis'}
					<AiCliSettings />
				{:else if activeSection === 'rate-limits'}
					<RateLimitSettings />
				{:else if activeSection === 'notifications'}
					<NotificationSettings />
				{:else if activeSection === 'keybindings'}
					<KeybindingSettings />
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		border: none;
		cursor: default;
		padding: 0;
	}

	.modal {
		background: var(--bg-surface);
		border-radius: var(--radius-container);
		width: 100%;
		max-width: 720px;
		max-height: 85vh;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		cursor: auto;
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 20px;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
	}

	.close-btn:hover {
		color: var(--text-primary);
	}

	.modal-content {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.sidebar-nav {
		width: 160px;
		flex-shrink: 0;
		padding: 12px;
		border-right: 1px solid var(--border-color);
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.nav-item {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 13px;
		padding: 8px 12px;
		border-radius: var(--radius-button);
		cursor: pointer;
		text-align: left;
		transition: color 0.15s, background 0.15s;
	}

	.nav-item:hover {
		color: var(--text-primary);
		background: var(--bg-primary);
	}

	.nav-item.active {
		color: var(--text-primary);
		background: var(--bg-primary);
	}

	.section-content {
		flex: 1;
		padding: 20px;
		overflow-y: auto;
	}
</style>
