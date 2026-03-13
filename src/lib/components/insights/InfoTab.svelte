<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { appStore } from '$lib/stores/app.svelte';

	interface SystemInfo {
		os: string;
		arch: string;
		hostname: string;
	}

	interface CliStatus {
		name: string;
		command: string;
		installed: boolean;
	}

	const AI_CLIS = [
		{ name: 'Claude Code', command: 'claude' },
		{ name: 'GitHub Copilot', command: 'gh' },
		{ name: 'Aider', command: 'aider' },
		{ name: 'Cursor', command: 'cursor' },
	];

	let systemInfo = $state<SystemInfo | null>(null);
	let cliStatuses = $state<CliStatus[]>([]);
	let loading = $state(true);

	let activeWorkspaceCount = $derived(appStore.workspaces.length);
	let activeSessionCount = $derived(
		appStore.workspaces.reduce((acc, ws) => acc + ws.sessions.length, 0)
	);

	async function loadSystemInfo() {
		try {
			systemInfo = await invoke<SystemInfo>('get_system_info');
		} catch {
			systemInfo = { os: 'unknown', arch: 'unknown', hostname: 'unknown' };
		}
	}

	async function loadCliStatuses() {
		const results = await Promise.all(
			AI_CLIS.map(async (cli) => {
				let installed = false;
				try {
					installed = await invoke<boolean>('check_cli_exists', { command: cli.command });
				} catch {
					installed = false;
				}
				return { name: cli.name, command: cli.command, installed };
			})
		);
		cliStatuses = results;
	}

	onMount(async () => {
		await Promise.all([loadSystemInfo(), loadCliStatuses()]);
		loading = false;
	});
</script>

<div class="info-tab">
	{#if loading}
		<div class="loading-state">Loading system info...</div>
	{:else}
		<section class="card">
			<h3>Application</h3>
			<div class="info-grid">
				<span class="info-label">Version</span>
				<span class="info-value mono">0.1.0</span>
			</div>
		</section>

		{#if systemInfo}
			<section class="card">
				<h3>System</h3>
				<div class="info-grid">
					<span class="info-label">OS</span>
					<span class="info-value mono">{systemInfo.os}</span>
					<span class="info-label">Architecture</span>
					<span class="info-value mono">{systemInfo.arch}</span>
					<span class="info-label">Hostname</span>
					<span class="info-value mono">{systemInfo.hostname}</span>
				</div>
			</section>
		{/if}

		<section class="card">
			<h3>Workspaces</h3>
			<div class="info-grid">
				<span class="info-label">Active Workspaces</span>
				<span class="info-value mono">{activeWorkspaceCount}</span>
				<span class="info-label">Total Sessions</span>
				<span class="info-value mono">{activeSessionCount}</span>
			</div>
		</section>

		<section class="card">
			<h3>AI CLIs</h3>
			<div class="cli-list">
				{#each cliStatuses as cli (cli.command)}
					<div class="cli-item">
						<span class="cli-name">{cli.name}</span>
						<span class="cli-command mono">{cli.command}</span>
						<span class="cli-status" class:installed={cli.installed}>
							{cli.installed ? 'Installed' : 'Not found'}
						</span>
					</div>
				{/each}
			</div>
		</section>
	{/if}
</div>

<style>
	.info-tab {
		padding: 20px 24px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 720px;
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

	.info-grid {
		display: grid;
		grid-template-columns: 160px 1fr;
		gap: 8px 16px;
		font-size: 13px;
	}

	.info-label {
		color: var(--text-secondary);
	}

	.info-value {
		color: var(--text-primary);
	}

	.mono {
		font-family: 'JetBrains Mono', monospace;
	}

	.cli-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.cli-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 10px;
		background: rgba(255, 255, 255, 0.03);
		border-radius: var(--radius-button);
		border: 1px solid rgba(255, 255, 255, 0.04);
		font-size: 13px;
	}

	.cli-name {
		color: var(--text-primary);
		min-width: 120px;
	}

	.cli-command {
		color: var(--text-secondary);
		font-size: 12px;
		flex: 1;
	}

	.cli-status {
		font-size: 11px;
		padding: 2px 8px;
		border-radius: 10px;
		background: rgba(239, 68, 68, 0.15);
		color: #ef4444;
	}

	.cli-status.installed {
		background: rgba(16, 163, 127, 0.15);
		color: var(--accent);
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 200px;
		color: var(--text-secondary);
		font-size: 13px;
	}
</style>
