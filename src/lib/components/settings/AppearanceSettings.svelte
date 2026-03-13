<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';

	function updateAccentColor(e: Event) {
		settingsStore.appearance.accentColor = (e.target as HTMLInputElement).value;
		settingsStore.applyAppearance();
		settingsStore.save();
	}

	function updateFontUi(e: Event) {
		settingsStore.appearance.fontUi = (e.target as HTMLInputElement).value;
		settingsStore.applyAppearance();
		settingsStore.save();
	}

	function updateFontCode(e: Event) {
		settingsStore.appearance.fontCode = (e.target as HTMLInputElement).value;
		settingsStore.applyAppearance();
		settingsStore.save();
	}

	function adjustFontSize(delta: number) {
		const next = settingsStore.appearance.fontSize + delta;
		if (next >= 10 && next <= 24) {
			settingsStore.appearance.fontSize = next;
			settingsStore.applyAppearance();
			settingsStore.save();
		}
	}

	function updateFontSize(e: Event) {
		const val = parseInt((e.target as HTMLInputElement).value, 10);
		if (val >= 10 && val <= 24) {
			settingsStore.appearance.fontSize = val;
			settingsStore.applyAppearance();
			settingsStore.save();
		}
	}
</script>

<div class="section">
	<h3 class="section-title">Appearance</h3>

	<div class="field">
		<label class="field-label" for="accent-color">Accent Color</label>
		<div class="color-row">
			<input
				id="accent-color"
				type="color"
				value={settingsStore.appearance.accentColor}
				oninput={updateAccentColor}
				class="color-input"
			/>
			<span class="color-value">{settingsStore.appearance.accentColor}</span>
		</div>
	</div>

	<div class="field">
		<label class="field-label" for="font-ui">UI Font</label>
		<input
			id="font-ui"
			type="text"
			value={settingsStore.appearance.fontUi}
			oninput={updateFontUi}
			class="text-input"
		/>
	</div>

	<div class="field">
		<label class="field-label" for="font-code">Code Font</label>
		<input
			id="font-code"
			type="text"
			value={settingsStore.appearance.fontCode}
			oninput={updateFontCode}
			class="text-input"
		/>
	</div>

	<div class="field">
		<label class="field-label" for="font-size">Font Size</label>
		<div class="stepper-row">
			<button class="stepper-btn" onclick={() => adjustFontSize(-1)}>-</button>
			<input
				id="font-size"
				type="number"
				min="10"
				max="24"
				value={settingsStore.appearance.fontSize}
				oninput={updateFontSize}
				class="stepper-input"
			/>
			<button class="stepper-btn" onclick={() => adjustFontSize(1)}>+</button>
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
		width: 100%;
	}

	.text-input:focus {
		border-color: var(--accent);
	}

	.color-row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.color-input {
		width: 36px;
		height: 36px;
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		background: none;
		cursor: pointer;
		padding: 2px;
	}

	.color-value {
		font-size: 13px;
		color: var(--text-secondary);
		font-family: monospace;
	}

	.stepper-row {
		display: flex;
		align-items: center;
		gap: 0;
		width: fit-content;
	}

	.stepper-btn {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		color: var(--text-primary);
		width: 32px;
		height: 34px;
		font-size: 16px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.stepper-btn:first-child {
		border-radius: var(--radius-button) 0 0 var(--radius-button);
	}

	.stepper-btn:last-child {
		border-radius: 0 var(--radius-button) var(--radius-button) 0;
	}

	.stepper-btn:hover {
		background: var(--bg-surface);
	}

	.stepper-input {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-left: none;
		border-right: none;
		color: var(--text-primary);
		width: 50px;
		height: 34px;
		text-align: center;
		font-size: 13px;
		appearance: textfield;
		-moz-appearance: textfield;
	}

	.stepper-input::-webkit-inner-spin-button,
	.stepper-input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
