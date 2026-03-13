<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { recentPathsStore } from '$lib/stores/recent-paths.svelte';

	interface Props {
		value: string;
		onchange: (path: string) => void;
	}

	let { value, onchange }: Props = $props();
	let showRecent = $state(false);

	function formatRelativeTime(isoDate: string): string {
		const diff = Date.now() - new Date(isoDate).getTime();
		const minutes = Math.floor(diff / 60000);
		const hours = Math.floor(diff / 3600000);
		const days = Math.floor(diff / 86400000);

		if (minutes < 1) return 'just now';
		if (minutes < 60) return `${minutes}m ago`;
		if (hours < 24) return `${hours}h ago`;
		if (days === 1) return 'yesterday';
		return `${days}d ago`;
	}

	async function browse() {
		const selected = await open({ directory: true });
		if (selected) {
			onchange(selected);
		}
	}

	function selectRecent(path: string) {
		onchange(path);
		showRecent = false;
	}

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		onchange(target.value);
	}

	function handleBackdropClick() {
		showRecent = false;
	}

	let recentPaths = $derived(recentPathsStore.getPaths());
</script>

{#if showRecent}
	<button class="dropdown-backdrop" onclick={handleBackdropClick} aria-label="Close dropdown"></button>
{/if}

<div class="path-picker">
	<div class="input-row">
		<span class="folder-icon">&#128193;</span>
		<input
			type="text"
			{value}
			oninput={handleInput}
			placeholder="/path/to/project"
			class="path-input"
		/>
		<button class="picker-btn" onclick={browse} title="Browse">
			&#128194;
		</button>
		<button class="picker-btn recent-btn" onclick={() => (showRecent = !showRecent)}>
			Recent &#9662;
		</button>
	</div>

	{#if showRecent && recentPaths.length > 0}
		<div class="recent-dropdown" role="listbox" aria-label="Recent paths">
			{#each recentPaths as entry}
				<button class="recent-item" role="option" aria-selected={entry.path === value} onclick={() => selectRecent(entry.path)}>
					<span class="recent-path">{entry.path}</span>
					<span class="recent-time">{formatRelativeTime(entry.lastUsed)}</span>
				</button>
			{/each}
		</div>
	{/if}

	{#if showRecent && recentPaths.length === 0}
		<div class="recent-dropdown">
			<div class="recent-empty">No recent paths</div>
		</div>
	{/if}
</div>

<style>
	.path-picker {
		position: relative;
	}

	.input-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.folder-icon {
		font-size: 16px;
		flex-shrink: 0;
	}

	.path-input {
		flex: 1;
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 8px 10px;
		font-size: 13px;
	}

	.path-input:focus {
		border-color: var(--accent);
	}

	.picker-btn {
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		padding: 7px 10px;
		font-size: 13px;
		cursor: pointer;
		white-space: nowrap;
	}

	.picker-btn:hover {
		border-color: var(--accent);
	}

	.recent-dropdown {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		z-index: 100;
		max-height: 240px;
		overflow-y: auto;
	}

	.recent-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		color: var(--text-primary);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
	}

	.recent-item:hover {
		background: var(--bg-surface);
	}

	.recent-path {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		margin-right: 12px;
	}

	.recent-time {
		color: var(--text-secondary);
		font-size: 11px;
		flex-shrink: 0;
	}

	.recent-empty {
		padding: 12px;
		color: var(--text-secondary);
		font-size: 12px;
		text-align: center;
	}

	.dropdown-backdrop {
		position: fixed;
		inset: 0;
		background: transparent;
		border: none;
		z-index: 99;
		cursor: default;
	}
</style>
