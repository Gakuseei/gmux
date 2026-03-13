<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';

	function updateShell(e: Event) {
		settingsStore.terminal.defaultShell = (e.target as HTMLInputElement).value;
		settingsStore.save();
	}

	function updateScrollback(e: Event) {
		const val = parseInt((e.target as HTMLInputElement).value, 10);
		if (val > 0) {
			settingsStore.terminal.scrollbackLines = val;
			settingsStore.save();
		}
	}

	function setCursorStyle(style: 'block' | 'beam' | 'underline') {
		settingsStore.terminal.cursorStyle = style;
		settingsStore.save();
	}
</script>

<div class="section">
	<h3 class="section-title">Terminal</h3>

	<div class="field">
		<label class="field-label" for="default-shell">Default Shell</label>
		<input
			id="default-shell"
			type="text"
			value={settingsStore.terminal.defaultShell}
			oninput={updateShell}
			class="text-input"
		/>
	</div>

	<div class="field">
		<label class="field-label" for="scrollback-lines">Scrollback Lines</label>
		<input
			id="scrollback-lines"
			type="number"
			min="1000"
			step="1000"
			value={settingsStore.terminal.scrollbackLines}
			oninput={updateScrollback}
			class="text-input"
		/>
	</div>

	<div class="field">
		<span class="field-label">Cursor Style</span>
		<div class="radio-group">
			{#each ['block', 'beam', 'underline'] as style}
				<button
					class="radio-btn"
					class:active={settingsStore.terminal.cursorStyle === style}
					onclick={() => setCursorStyle(style as 'block' | 'beam' | 'underline')}
				>
					<span class="cursor-preview" data-style={style}></span>
					{style}
				</button>
			{/each}
		</div>
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

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
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
		outline: none;
		width: 100%;
	}

	.text-input:focus {
		border-color: var(--accent);
	}

	.radio-group {
		display: flex;
		gap: 8px;
	}

	.radio-btn {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-secondary);
		padding: 8px 14px;
		font-size: 13px;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 8px;
		text-transform: capitalize;
	}

	.radio-btn:hover {
		border-color: var(--text-secondary);
		color: var(--text-primary);
	}

	.radio-btn.active {
		border-color: var(--accent);
		color: var(--text-primary);
		background: var(--bg-surface);
	}

	.cursor-preview {
		width: 8px;
		height: 16px;
		display: inline-block;
	}

	.cursor-preview[data-style='block'] {
		background: var(--text-primary);
	}

	.cursor-preview[data-style='beam'] {
		border-left: 2px solid var(--text-primary);
	}

	.cursor-preview[data-style='underline'] {
		border-bottom: 2px solid var(--text-primary);
		align-self: flex-end;
	}
</style>
