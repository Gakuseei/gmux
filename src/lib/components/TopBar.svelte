<script lang="ts">
	import { appStore } from '$lib/stores/app.svelte';
</script>

<header class="topbar">
	<div class="left">
		<span class="logo">gmux</span>
	</div>

	<div class="center"></div>

	<div class="right">
		<button
			class="tab-btn"
			class:active={appStore.activeView === 'terminals'}
			onclick={() => (appStore.activeView = 'terminals')}
		>
			Terminals
		</button>
		<button
			class="tab-btn"
			class:active={appStore.activeView === 'insights'}
			onclick={() => (appStore.activeView = 'insights')}
		>
			Insights
		</button>
		<button class="icon-btn" aria-label="Settings" onclick={() => (appStore.showSettings = true)}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
				<circle cx="12" cy="12" r="3" />
			</svg>
		</button>
	</div>
</header>

<style>
	.topbar {
		display: flex;
		align-items: center;
		height: 40px;
		padding: 0 12px;
		background: var(--bg-primary);
		border-bottom: 1px solid var(--border-color);
		user-select: none;
	}

	.left {
		display: flex;
		align-items: center;
	}

	.logo {
		font-weight: 700;
		font-size: 13px;
		letter-spacing: 0.5px;
	}

	.center {
		flex: 1;
	}

	.right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.tab-btn {
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-secondary);
		font-size: 12px;
		padding: 8px 10px;
		cursor: pointer;
		font-family: inherit;
		transition: color 0.15s, border-color 0.15s;
	}

	.tab-btn:hover {
		color: var(--text-primary);
	}

	.tab-btn.active {
		color: var(--text-primary);
		border-bottom-color: var(--accent);
	}

	.icon-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		padding: 6px;
		border-radius: var(--radius-button);
		display: flex;
		align-items: center;
		justify-content: center;
		transition: color 0.15s, background 0.15s;
	}

	.icon-btn:hover {
		color: var(--text-primary);
		background: var(--bg-surface);
	}
</style>
