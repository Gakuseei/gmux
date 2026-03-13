<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';

	type Provider = 'claude' | 'codex' | 'gemini';

	const DAYS = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];

	function updateFiveHour(provider: Provider, e: Event) {
		settingsStore.rateLimits[provider].fiveHourLimit = parseInt((e.target as HTMLInputElement).value, 10) || 0;
		settingsStore.save();
	}

	function updateWeekly(provider: Provider, e: Event) {
		settingsStore.rateLimits[provider].weeklyLimit = parseInt((e.target as HTMLInputElement).value, 10) || 0;
		settingsStore.save();
	}

	function updateResetDay(provider: Provider, e: Event) {
		settingsStore.rateLimits[provider].resetDay = (e.target as HTMLSelectElement).value;
		settingsStore.save();
	}

	function updateResetHour(provider: Provider, e: Event) {
		const val = parseInt((e.target as HTMLInputElement).value, 10);
		if (val >= 0 && val <= 23) {
			settingsStore.rateLimits[provider].resetHour = val;
			settingsStore.save();
		}
	}
</script>

<div class="section">
	<h3 class="section-title">Rate Limits</h3>

	{#each ['claude', 'codex', 'gemini'] as provider}
		{@const p = provider as Provider}
		<div class="provider-block">
			<h4 class="provider-name">{provider}</h4>

			<div class="fields-grid">
				<div class="field">
					<label class="field-label" for="{provider}-5h">5h Window Limit</label>
					<input
						id="{provider}-5h"
						type="number"
						min="0"
						step="100000"
						value={settingsStore.rateLimits[p].fiveHourLimit}
						oninput={(e) => updateFiveHour(p, e)}
						class="text-input"
					/>
				</div>

				<div class="field">
					<label class="field-label" for="{provider}-weekly">Weekly Limit</label>
					<input
						id="{provider}-weekly"
						type="number"
						min="0"
						step="100000"
						value={settingsStore.rateLimits[p].weeklyLimit}
						oninput={(e) => updateWeekly(p, e)}
						class="text-input"
					/>
				</div>

				<div class="field">
					<label class="field-label" for="{provider}-day">Reset Day</label>
					<select
						id="{provider}-day"
						value={settingsStore.rateLimits[p].resetDay}
						onchange={(e) => updateResetDay(p, e)}
						class="select-input"
					>
						{#each DAYS as day}
							<option value={day}>{day}</option>
						{/each}
					</select>
				</div>

				<div class="field">
					<label class="field-label" for="{provider}-hour">Reset Hour (0-23)</label>
					<input
						id="{provider}-hour"
						type="number"
						min="0"
						max="23"
						value={settingsStore.rateLimits[p].resetHour}
						oninput={(e) => updateResetHour(p, e)}
						class="text-input"
					/>
				</div>
			</div>
		</div>
	{/each}
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

	.provider-block {
		padding: 12px;
		background: var(--bg-primary);
		border-radius: var(--radius-button);
		border: 1px solid var(--border-color);
	}

	.provider-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		text-transform: capitalize;
		margin-bottom: 12px;
	}

	.fields-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.field-label {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.text-input,
	.select-input {
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 6px 8px;
		font-size: 13px;
		width: 100%;
	}

	.text-input:focus,
	.select-input:focus {
		border-color: var(--accent);
	}

	.select-input {
		cursor: pointer;
		text-transform: capitalize;
	}
</style>
