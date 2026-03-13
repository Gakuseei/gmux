<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';

	function toggleDesktop() {
		settingsStore.notifications.desktopEnabled = !settingsStore.notifications.desktopEnabled;
		settingsStore.save();
	}

	function toggleSound() {
		settingsStore.notifications.soundEnabled = !settingsStore.notifications.soundEnabled;
		settingsStore.save();
	}

	function addPattern() {
		settingsStore.notifications.customPatterns.push('');
		settingsStore.save();
	}

	function removePattern(index: number) {
		settingsStore.notifications.customPatterns.splice(index, 1);
		settingsStore.save();
	}

	function updatePattern(index: number, e: Event) {
		settingsStore.notifications.customPatterns[index] = (e.target as HTMLInputElement).value;
		settingsStore.save();
	}
</script>

<div class="section">
	<h3 class="section-title">Notifications</h3>

	<div class="toggle-row">
		<span class="toggle-label">Desktop Notifications</span>
		<button
			class="toggle-btn"
			class:active={settingsStore.notifications.desktopEnabled}
			onclick={toggleDesktop}
			aria-label="Toggle desktop notifications"
		>
			<span class="toggle-track">
				<span class="toggle-thumb"></span>
			</span>
		</button>
	</div>

	<div class="toggle-row">
		<span class="toggle-label">Sound</span>
		<button
			class="toggle-btn"
			class:active={settingsStore.notifications.soundEnabled}
			onclick={toggleSound}
			aria-label="Toggle sound"
		>
			<span class="toggle-track">
				<span class="toggle-thumb"></span>
			</span>
		</button>
	</div>

	<div class="patterns-section">
		<div class="patterns-header">
			<span class="field-label">Custom Patterns (regex)</span>
			<button class="add-btn" onclick={addPattern}>+ Add</button>
		</div>

		{#each settingsStore.notifications.customPatterns as pattern, i}
			<div class="pattern-row">
				<input
					type="text"
					value={pattern}
					oninput={(e) => updatePattern(i, e)}
					placeholder="e.g. error|warning"
					class="text-input pattern-input"
				/>
				<button class="remove-btn" aria-label="Remove pattern" onclick={() => removePattern(i)}>&times;</button>
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

	.toggle-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 0;
	}

	.toggle-label {
		font-size: 13px;
		color: var(--text-primary);
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

	.patterns-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 8px;
		padding-top: 16px;
		border-top: 1px solid var(--border-color);
	}

	.patterns-header {
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

	.pattern-row {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.text-input {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 8px 10px;
		font-size: 13px;
	}

	.text-input:focus {
		border-color: var(--accent);
	}

	.pattern-input {
		flex: 1;
		font-family: monospace;
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
