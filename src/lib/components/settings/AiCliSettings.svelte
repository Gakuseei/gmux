<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';

	function updatePath(key: 'claude' | 'codex' | 'gemini', e: Event) {
		settingsStore.aiClis[key].path = (e.target as HTMLInputElement).value;
		settingsStore.save();
	}

	function toggleEnabled(key: 'claude' | 'codex' | 'gemini') {
		settingsStore.aiClis[key].enabled = !settingsStore.aiClis[key].enabled;
		settingsStore.save();
	}

	function addCustom() {
		settingsStore.aiClis.custom.push({ name: '', command: '' });
		settingsStore.save();
	}

	function removeCustom(index: number) {
		settingsStore.aiClis.custom.splice(index, 1);
		settingsStore.save();
	}

	function updateCustomName(index: number, e: Event) {
		settingsStore.aiClis.custom[index].name = (e.target as HTMLInputElement).value;
		settingsStore.save();
	}

	function updateCustomCommand(index: number, e: Event) {
		settingsStore.aiClis.custom[index].command = (e.target as HTMLInputElement).value;
		settingsStore.save();
	}
</script>

<div class="section">
	<h3 class="section-title">AI CLIs</h3>

	{#each ['claude', 'codex', 'gemini'] as key}
		<div class="cli-row">
			<div class="cli-header">
				<span class="cli-name">{key}</span>
				<button
					class="toggle-btn"
					class:active={settingsStore.aiClis[key as 'claude' | 'codex' | 'gemini'].enabled}
					onclick={() => toggleEnabled(key as 'claude' | 'codex' | 'gemini')}
					aria-label="Toggle {key}"
				>
					<span class="toggle-track">
						<span class="toggle-thumb"></span>
					</span>
				</button>
			</div>
			<input
				type="text"
				value={settingsStore.aiClis[key as 'claude' | 'codex' | 'gemini'].path}
				oninput={(e) => updatePath(key as 'claude' | 'codex' | 'gemini', e)}
				placeholder="Path to {key}"
				class="text-input"
			/>
		</div>
	{/each}

	<div class="custom-section">
		<div class="custom-header">
			<span class="field-label">Custom Commands</span>
			<button class="add-btn" onclick={addCustom}>+ Add</button>
		</div>

		{#each settingsStore.aiClis.custom as item, i}
			<div class="custom-row">
				<input
					type="text"
					value={item.name}
					oninput={(e) => updateCustomName(i, e)}
					placeholder="Name"
					class="text-input custom-input"
				/>
				<input
					type="text"
					value={item.command}
					oninput={(e) => updateCustomCommand(i, e)}
					placeholder="Command"
					class="text-input custom-input"
				/>
				<button class="remove-btn" onclick={() => removeCustom(i)}>&times;</button>
			</div>
		{/each}
	</div>
</div>

<style>
	.section {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.section-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		margin-bottom: 4px;
	}

	.field-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.text-input {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 8px 10px;
		font-size: 13px;
		width: 100%;
	}

	.text-input:focus {
		border-color: var(--accent);
	}

	.cli-row {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.cli-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.cli-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		text-transform: capitalize;
	}

	.toggle-btn {
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	.toggle-track {
		display: block;
		width: 36px;
		height: 20px;
		background: var(--border-color);
		border-radius: 10px;
		position: relative;
		transition: background 0.2s;
	}

	.toggle-btn.active .toggle-track {
		background: var(--accent);
	}

	.toggle-thumb {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
	}

	.toggle-btn.active .toggle-thumb {
		transform: translateX(16px);
	}

	.custom-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 8px;
		padding-top: 16px;
		border-top: 1px solid var(--border-color);
	}

	.custom-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.add-btn {
		background: none;
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-secondary);
		padding: 4px 10px;
		font-size: 12px;
		cursor: pointer;
	}

	.add-btn:hover {
		color: var(--text-primary);
		border-color: var(--text-secondary);
	}

	.custom-row {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.custom-input {
		flex: 1;
	}

	.remove-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 18px;
		cursor: pointer;
		padding: 4px 6px;
		line-height: 1;
		flex-shrink: 0;
	}

	.remove-btn:hover {
		color: #ef4444;
	}
</style>
