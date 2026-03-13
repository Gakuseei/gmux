<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';

	const ACTION_LABELS: Record<string, string> = {
		splitHorizontal: 'Split Horizontal',
		splitVertical: 'Split Vertical',
		closePane: 'Close Pane',
		newWorkspace: 'New Workspace',
		newTerminal: 'New Terminal',
		toggleSidebar: 'Toggle Sidebar',
		search: 'Search',
		nextPane: 'Next Pane',
		prevPane: 'Previous Pane',
	};

	let recordingKey = $state<string | null>(null);

	function startRecording(key: string) {
		recordingKey = key;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!recordingKey) return;
		e.preventDefault();
		e.stopPropagation();

		if (e.key === 'Escape') {
			recordingKey = null;
			return;
		}

		const parts: string[] = [];
		if (e.ctrlKey) parts.push('Ctrl');
		if (e.shiftKey) parts.push('Shift');
		if (e.altKey) parts.push('Alt');
		if (e.metaKey) parts.push('Meta');

		const key = e.key;
		if (!['Control', 'Shift', 'Alt', 'Meta'].includes(key)) {
			if (key === 'Tab') {
				parts.push('Tab');
			} else if (key === ' ') {
				parts.push('Space');
			} else if (key.length === 1) {
				parts.push(key.toUpperCase());
			} else {
				parts.push(key);
			}

			const binding = parts.join('+');
			(settingsStore.keybindings as Record<string, string>)[recordingKey] = binding;
			settingsStore.save();
			recordingKey = null;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="section">
	<h3 class="section-title">Keybindings</h3>
	<p class="section-hint">Click a keybinding to record a new one. Press Escape to cancel.</p>

	<div class="bindings-list">
		{#each Object.entries(ACTION_LABELS) as [key, label]}
			<div class="binding-row">
				<span class="action-label">{label}</span>
				<button
					class="keybinding-btn"
					class:recording={recordingKey === key}
					onclick={() => startRecording(key)}
				>
					{#if recordingKey === key}
						<span class="recording-text">Press keys...</span>
					{:else}
						<kbd>{(settingsStore.keybindings as Record<string, string>)[key]}</kbd>
					{/if}
				</button>
			</div>
		{/each}
	</div>
</div>

<style>
	.section {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.section-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.section-hint {
		font-size: 12px;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}

	.bindings-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.binding-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 10px;
		border-radius: var(--radius-button);
	}

	.binding-row:hover {
		background: var(--bg-primary);
	}

	.action-label {
		font-size: 13px;
		color: var(--text-primary);
	}

	.keybinding-btn {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 4px 10px;
		font-size: 12px;
		cursor: pointer;
		min-width: 120px;
		text-align: center;
	}

	.keybinding-btn:hover {
		border-color: var(--text-secondary);
	}

	.keybinding-btn.recording {
		border-color: var(--accent);
		background: var(--bg-surface);
	}

	.recording-text {
		color: var(--accent);
		font-style: italic;
	}

	kbd {
		font-family: inherit;
		font-size: 12px;
	}
</style>
