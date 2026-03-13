<script lang="ts">
	interface AgentConfig {
		type: 'claude' | 'codex' | 'gemini' | 'shell';
		label: string;
		command: string;
		count: number;
		bypassPermissions?: boolean;
	}

	interface Props {
		totalSlots: number;
		agents: AgentConfig[];
		onchange: (agents: AgentConfig[]) => void;
	}

	let { totalSlots, agents, onchange }: Props = $props();

	let assignedSlots = $derived(agents.reduce((sum, a) => sum + a.count, 0));
	let remainingSlots = $derived(totalSlots - assignedSlots);

	function increment(index: number) {
		if (assignedSlots >= totalSlots) return;
		const updated = agents.map((a, i) =>
			i === index ? { ...a, count: a.count + 1 } : { ...a }
		);
		onchange(updated);
	}

	function decrement(index: number) {
		if (agents[index].count <= 0) return;
		const updated = agents.map((a, i) =>
			i === index ? { ...a, count: a.count - 1 } : { ...a }
		);
		onchange(updated);
	}

	function toggleBypass(index: number) {
		const updated = agents.map((a, i) =>
			i === index ? { ...a, bypassPermissions: !a.bypassPermissions } : { ...a }
		);
		onchange(updated);
	}
</script>

<div class="agent-picker">
	<div class="section-header">Agents (optional — leave all at 0 for plain terminals)</div>

	<div class="agent-list">
		{#each agents as agent, i}
			<div class="agent-row">
				<span class="agent-name">{agent.label}</span>
				<div class="counter">
					<button
						class="counter-btn"
						onclick={() => decrement(i)}
						disabled={agent.count <= 0}
					>-</button>
					<span class="counter-value">{agent.count}</span>
					<button
						class="counter-btn"
						onclick={() => increment(i)}
						disabled={assignedSlots >= totalSlots}
					>+</button>
				</div>
				{#if agent.type === 'claude'}
					<label class="bypass-label">
						<input
							type="checkbox"
							checked={agent.bypassPermissions ?? false}
							onchange={() => toggleBypass(i)}
						/>
						<span>bypass permissions</span>
					</label>
				{/if}
			</div>
		{/each}
	</div>

	<div class="slot-info">
		<span>{assignedSlots}/{totalSlots} slots assigned</span>
		{#if remainingSlots > 0}
			<span class="remaining">Remaining {remainingSlots} slot{remainingSlots === 1 ? '' : 's'} will be plain shells</span>
		{/if}
	</div>
</div>

<style>
	.agent-picker {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.section-header {
		color: var(--text-secondary);
		font-size: 12px;
	}

	.agent-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.agent-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.agent-name {
		width: 100px;
		font-size: 13px;
		color: var(--text-primary);
	}

	.counter {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.counter-btn {
		width: 26px;
		height: 26px;
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		font-size: 14px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		line-height: 1;
	}

	.counter-btn:hover:not(:disabled) {
		border-color: var(--accent);
	}

	.counter-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.counter-value {
		width: 24px;
		text-align: center;
		font-size: 13px;
		font-variant-numeric: tabular-nums;
	}

	.bypass-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--text-secondary);
		cursor: pointer;
		margin-left: 4px;
	}

	.bypass-label input {
		accent-color: var(--accent);
	}

	.slot-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.remaining {
		font-size: 11px;
		opacity: 0.7;
	}
</style>
