<script lang="ts">
	import { onMount } from 'svelte';
	import { usageStore } from '$lib/stores/usage.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';

	let rates = $derived(settingsStore.costRates.claude);
	let fiveHourLimit = $derived(settingsStore.rateLimits.claude.fiveHourLimit);
	let weeklyLimit = $derived(settingsStore.rateLimits.claude.weeklyLimit);

	function cost(tokens: number, rate: number): number {
		return (tokens / 1_000_000) * rate;
	}

	function formatCost(c: number): string {
		return c < 0.01 && c > 0 ? '<$0.01' : '$' + c.toFixed(2);
	}

	function formatTokens(n: number): string {
		if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
		if (n >= 1_000) return (n / 1_000).toFixed(1) + 'k';
		return String(n);
	}

	function totalCost(input: number, output: number, cacheRead: number, cacheWrite: number): number {
		return cost(input, rates.input) + cost(output, rates.output) + cost(cacheRead, rates.cacheRead) + cost(cacheWrite, rates.cacheWrite);
	}

	function fiveHourUsage(): number {
		if (!usageStore.data) return 0;
		const cutoff = Date.now() - 5 * 60 * 60 * 1000;
		return usageStore.data.sessions
			.filter((s) => new Date(s.timestamp).getTime() > cutoff)
			.reduce((acc, s) => acc + s.input_tokens + s.output_tokens, 0);
	}

	function fiveHourResetMinutes(): number {
		if (!usageStore.data || usageStore.data.sessions.length === 0) return 0;
		const cutoff = Date.now() - 5 * 60 * 60 * 1000;
		const relevantSessions = usageStore.data.sessions.filter(
			(s) => new Date(s.timestamp).getTime() > cutoff
		);
		if (relevantSessions.length === 0) return 0;
		const oldest = Math.min(...relevantSessions.map((s) => new Date(s.timestamp).getTime()));
		const resetAt = oldest + 5 * 60 * 60 * 1000;
		return Math.max(0, Math.ceil((resetAt - Date.now()) / 60_000));
	}

	function weeklyUsage(): number {
		if (!usageStore.data) return 0;
		return usageStore.data.sessions.reduce((acc, s) => acc + s.input_tokens + s.output_tokens, 0);
	}

	function progressPercent(used: number, limit: number): number {
		return Math.min(100, (used / limit) * 100);
	}

	function progressColor(pct: number): string {
		if (pct >= 90) return 'var(--color-error, #ef4444)';
		if (pct >= 70) return 'var(--color-warning, #f59e0b)';
		return 'var(--accent, #10a37f)';
	}

	function formatMinutes(mins: number): string {
		const h = Math.floor(mins / 60);
		const m = mins % 60;
		if (h > 0) return `${h}h ${m}m`;
		return `${m}m`;
	}

	function shortSessionId(id: string): string {
		return id.length > 12 ? id.slice(0, 12) + '...' : id;
	}

	const periods: Array<'today' | 'weekly' | 'monthly'> = ['today', 'weekly', 'monthly'];

	onMount(() => {
		usageStore.refresh();
	});
</script>

<div class="usage-tab">
	<div class="header">
		<div class="period-toggle">
			{#each periods as p}
				<button
					class="period-btn"
					class:active={usageStore.period === p}
					onclick={() => usageStore.setPeriod(p)}
				>
					{p.charAt(0).toUpperCase() + p.slice(1)}
				</button>
			{/each}
		</div>
		<button class="refresh-btn" onclick={() => usageStore.refresh()} disabled={usageStore.loading}>
			{usageStore.loading ? 'Loading...' : 'Refresh'}
		</button>
	</div>

	{#if usageStore.data}
		{@const d = usageStore.data}
		<section class="card">
			<h3>Token Breakdown</h3>
			<table class="token-table">
				<thead>
					<tr>
						<th>Type</th>
						<th>Tokens</th>
						<th>Cost</th>
					</tr>
				</thead>
				<tbody>
					<tr>
						<td>Input</td>
						<td class="mono">{formatTokens(d.total_input)}</td>
						<td class="mono">{formatCost(cost(d.total_input, rates.input))}</td>
					</tr>
					<tr>
						<td>Output</td>
						<td class="mono">{formatTokens(d.total_output)}</td>
						<td class="mono">{formatCost(cost(d.total_output, rates.output))}</td>
					</tr>
					<tr>
						<td>Cache Read</td>
						<td class="mono">{formatTokens(d.total_cache_read)}</td>
						<td class="mono">{formatCost(cost(d.total_cache_read, rates.cacheRead))}</td>
					</tr>
					<tr>
						<td>Cache Write</td>
						<td class="mono">{formatTokens(d.total_cache_write)}</td>
						<td class="mono">{formatCost(cost(d.total_cache_write, rates.cacheWrite))}</td>
					</tr>
					<tr class="total-row">
						<td>Total</td>
						<td class="mono">{formatTokens(d.total_input + d.total_output + d.total_cache_read + d.total_cache_write)}</td>
						<td class="mono">{formatCost(totalCost(d.total_input, d.total_output, d.total_cache_read, d.total_cache_write))}</td>
					</tr>
				</tbody>
			</table>
		</section>

		<section class="card">
			<h3>Rate Limits</h3>
			<div class="rate-limit-item">
				<div class="rate-label">
					<span>5h Window</span>
					<span class="rate-stats">{formatTokens(fiveHourUsage())} / {formatTokens(fiveHourLimit)}</span>
				</div>
				<div class="progress-bar">
					<div class="progress-fill" style:width="{progressPercent(fiveHourUsage(), fiveHourLimit)}%" style:background={progressColor(progressPercent(fiveHourUsage(), fiveHourLimit))}></div>
				</div>
				{#if fiveHourResetMinutes() > 0}
					<span class="reset-text">Resets in {formatMinutes(fiveHourResetMinutes())}</span>
				{/if}
			</div>

			<div class="rate-limit-item">
				<div class="rate-label">
					<span>Weekly</span>
					<span class="rate-stats">{formatTokens(weeklyUsage())} / {formatTokens(weeklyLimit)}</span>
				</div>
				<div class="progress-bar">
					<div class="progress-fill" style:width="{progressPercent(weeklyUsage(), weeklyLimit)}%" style:background={progressColor(progressPercent(weeklyUsage(), weeklyLimit))}></div>
				</div>
			</div>
		</section>

		{#if d.sessions.length > 0}
			<section class="card">
				<h3>Sessions ({d.sessions.length})</h3>
				<div class="session-list">
					{#each d.sessions as session (session.session_id)}
						{@const sessionCost = totalCost(session.input_tokens, session.output_tokens, session.cache_read_tokens, session.cache_write_tokens)}
						<div class="session-item">
							<div class="session-header">
								<span class="session-id">{shortSessionId(session.session_id)}</span>
								<span class="session-cost">{formatCost(sessionCost)}</span>
							</div>
							<div class="session-tokens">
								<span>In: {formatTokens(session.input_tokens)}</span>
								<span>Out: {formatTokens(session.output_tokens)}</span>
								<span>CR: {formatTokens(session.cache_read_tokens)}</span>
								<span>CW: {formatTokens(session.cache_write_tokens)}</span>
							</div>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{:else if usageStore.loading}
		<div class="loading-state">Loading usage data...</div>
	{:else}
		<div class="empty-state">No usage data found</div>
	{/if}
</div>

<style>
	.usage-tab {
		padding: 20px 24px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 720px;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.period-toggle {
		display: flex;
		gap: 0;
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		overflow: hidden;
	}

	.period-btn {
		background: none;
		border: none;
		border-right: 1px solid var(--border-color);
		color: var(--text-secondary);
		font-size: 12px;
		font-family: inherit;
		padding: 6px 14px;
		cursor: pointer;
		transition: color 0.15s, background 0.15s;
	}

	.period-btn:last-child {
		border-right: none;
	}

	.period-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.04);
	}

	.period-btn.active {
		color: var(--text-primary);
		background: rgba(16, 163, 127, 0.2);
	}

	.refresh-btn {
		background: rgba(255, 255, 255, 0.06);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-secondary);
		font-size: 12px;
		font-family: inherit;
		padding: 6px 12px;
		cursor: pointer;
		transition: color 0.15s, background 0.15s;
	}

	.refresh-btn:hover:not(:disabled) {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.1);
	}

	.refresh-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.card {
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-container);
		padding: 16px;
	}

	.card h3 {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-secondary);
		margin-bottom: 12px;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.token-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	.token-table th {
		text-align: left;
		color: var(--text-secondary);
		font-weight: 500;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.3px;
		padding: 4px 0;
		border-bottom: 1px solid var(--border-color);
	}

	.token-table td {
		padding: 6px 0;
		color: var(--text-primary);
	}

	.token-table .mono {
		font-family: var(--font-code);
		text-align: right;
	}

	.token-table th:nth-child(2),
	.token-table th:nth-child(3) {
		text-align: right;
	}

	.total-row {
		border-top: 1px solid var(--border-color);
	}

	.total-row td {
		font-weight: 600;
		padding-top: 8px;
	}

	.rate-limit-item {
		margin-bottom: 14px;
	}

	.rate-limit-item:last-child {
		margin-bottom: 0;
	}

	.rate-label {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 12px;
		color: var(--text-primary);
		margin-bottom: 6px;
	}

	.rate-stats {
		font-family: var(--font-code);
		font-size: 11px;
		color: var(--text-secondary);
	}

	.progress-bar {
		height: 6px;
		background: rgba(255, 255, 255, 0.06);
		border-radius: 3px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.3s ease;
	}

	.reset-text {
		font-size: 11px;
		color: var(--text-secondary);
		margin-top: 4px;
		display: block;
	}

	.session-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-height: 400px;
		overflow-y: auto;
	}

	.session-list::-webkit-scrollbar {
		width: 4px;
	}

	.session-list::-webkit-scrollbar-thumb {
		background: rgba(255, 255, 255, 0.1);
		border-radius: 2px;
	}

	.session-item {
		padding: 8px 10px;
		background: rgba(255, 255, 255, 0.03);
		border-radius: var(--radius-button);
		border: 1px solid rgba(255, 255, 255, 0.04);
	}

	.session-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 4px;
	}

	.session-id {
		font-family: var(--font-code);
		font-size: 12px;
		color: var(--text-primary);
	}

	.session-cost {
		font-family: var(--font-code);
		font-size: 12px;
		color: var(--accent);
		font-weight: 600;
	}

	.session-tokens {
		display: flex;
		gap: 12px;
		font-size: 11px;
		color: var(--text-secondary);
		font-family: var(--font-code);
	}

	.loading-state,
	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 200px;
		color: var(--text-secondary);
		font-size: 13px;
	}
</style>
