<script lang="ts">
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import { WebLinksAddon } from '@xterm/addon-web-links';
	import { SearchAddon } from '@xterm/addon-search';
	import '@xterm/xterm/css/xterm.css';
	import { createPty, writePty, resizePty, killPty, saveScrollback, loadScrollback } from './terminal-bridge';
	import { appStore } from '$lib/stores/app.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { notifications } from '$lib/stores/notifications.svelte';
	import { statusStore } from '$lib/stores/status.svelte';
	import { detectNotification, createLineBuffer } from '$lib/utils/notification-detector';

	let {
		terminalId,
		shell,
		cwd,
		command,
		existingPtyId,
		onTitleChange,
		onData
	}: {
		terminalId: string;
		shell?: string;
		cwd: string;
		command?: string;
		existingPtyId?: string;
		onTitleChange?: (title: string) => void;
		onData?: (data: string) => void;
	} = $props();

	let compiledCustomPatterns = $derived.by(() => {
		return settingsStore.notifications.customPatterns
			.filter((s) => s.length > 0)
			.map((s, i) => {
				try {
					return { name: `custom-${i}`, regex: new RegExp(s) };
				} catch {
					return null;
				}
			})
			.filter((p): p is { name: string; regex: RegExp } => p !== null);
	});

	const lineBuffer = createLineBuffer((line) => {
		const customs = compiledCustomPatterns;
		const result = detectNotification(line, customs.length > 0 ? customs : undefined);
		if (result.matched && appStore.activeTerminalId !== terminalId) {
			notifications.notify(terminalId, result.pattern);
		}
		if (appStore.activeTerminalId === terminalId) {
			statusStore.updateFromLine(line);
		}
	});

	function handleContainerClick() {
		appStore.activeTerminalId = terminalId;
		notifications.clear(terminalId);
		statusStore.updateGitBranch(cwd);
	}

	let containerEl: HTMLDivElement | undefined = $state();
	let ptyId: string | null = $state(null);
	let isAlive: boolean = $state(false);
	let searchAddon: SearchAddon | null = $state(null);
	let showSearch = $state(false);
	let searchQuery = $state('');
	let searchInput: HTMLInputElement | undefined = $state();
	let spawnError: string | null = $state(null);

	function toggleSearch() {
		showSearch = !showSearch;
		if (showSearch) {
			requestAnimationFrame(() => searchInput?.focus());
		}
	}

	$effect(() => {
		const handler = () => {
			if (appStore.activeTerminalId === terminalId) {
				toggleSearch();
			}
		};
		window.addEventListener('gmux-toggle-search', handler);
		return () => window.removeEventListener('gmux-toggle-search', handler);
	});

	function handleSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			if (e.shiftKey) {
				searchAddon?.findPrevious(searchQuery);
			} else {
				searchAddon?.findNext(searchQuery);
			}
		}
		if (e.key === 'Escape') {
			showSearch = false;
		}
	}

	$effect(() => {
		if (!containerEl) return;

		let term: Terminal | null = null;
		let fitAddon: FitAddon | null = null;
		let resizeObserver: ResizeObserver | null = null;
		let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
		let disposed = false;

		const setup = async () => {
			const resolvedShell = shell || settingsStore.terminal.defaultShell || '/bin/bash';

			const computedStyle = getComputedStyle(document.documentElement);
			const bgColor = computedStyle.getPropertyValue('--bg-primary').trim() || '#171717';
			const fgColor = computedStyle.getPropertyValue('--text-primary').trim() || '#e5e5e5';
			const accentColor = computedStyle.getPropertyValue('--accent').trim() || '#10a37f';

			term = new Terminal({
				fontFamily: settingsStore.appearance.fontCode,
				fontSize: settingsStore.appearance.fontSize,
				theme: {
					background: bgColor,
					foreground: fgColor,
					cursor: accentColor,
					selectionBackground: `${accentColor}4d`
				},
				cursorBlink: true,
				cursorStyle: settingsStore.terminal.cursorStyle,
				scrollback: settingsStore.terminal.scrollbackLines,
				allowProposedApi: true
			});

			fitAddon = new FitAddon();
			const search = new SearchAddon();
			searchAddon = search;
			term.loadAddon(fitAddon);
			term.loadAddon(search);
			term.loadAddon(new WebLinksAddon());

			term.open(containerEl!);

			try {
				const { WebglAddon } = await import('@xterm/addon-webgl');
				term.loadAddon(new WebglAddon());
			} catch {
			}

			fitAddon.fit();

			if (onTitleChange) {
				term.onTitleChange(onTitleChange);
			}

			const decoder = new TextDecoder();

			if (existingPtyId) {
				ptyId = existingPtyId;
				isAlive = true;
				try {
					const saved = await loadScrollback(terminalId);
					if (saved && term) {
						term.write(saved);
					}
				} catch {
				}
			} else {
				try {
					let receivedFirstOutput = false;
					let pendingCommand = command ?? null;

					const id = await createPty(
						resolvedShell,
						cwd,
						term.cols,
						term.rows,
						(data) => {
							if (disposed || !term) return;
							term.write(data);
							const text = decoder.decode(data, { stream: true });
							lineBuffer(text);
							if (onData) {
								onData(text);
							}
							if (!receivedFirstOutput && pendingCommand) {
								receivedFirstOutput = true;
								const cmd = pendingCommand;
								pendingCommand = null;
								writePty(id, cmd + '\r');
							}
						},
						(_code) => {
							isAlive = false;
						}
					);

					if (disposed) {
						killPty(id);
						return;
					}

					ptyId = id;
					isAlive = true;
				} catch (e) {
					spawnError = String(e);
					return;
				}
			}

			term.onData((input) => {
				if (ptyId && isAlive) {
					writePty(ptyId, input);
				}
			});

			resizeObserver = new ResizeObserver(() => {
				if (resizeTimeout) clearTimeout(resizeTimeout);
				resizeTimeout = setTimeout(() => {
					if (disposed || !fitAddon || !term || !ptyId || !isAlive) return;
					fitAddon.fit();
					resizePty(ptyId, term.rows, term.cols);
				}, 50);
			});
			resizeObserver.observe(containerEl!);
		};

		setup();

		return () => {
			disposed = true;
			if (resizeTimeout) clearTimeout(resizeTimeout);
			if (resizeObserver) resizeObserver.disconnect();
			if (term) {
				const buffer = term.buffer.active;
				const lines: string[] = [];
				for (let i = 0; i < buffer.length; i++) {
					const line = buffer.getLine(i);
					if (line) lines.push(line.translateToString(true));
				}
				const content = lines.join('\n');
				if (content.trim().length > 0) {
					saveScrollback(terminalId, content).catch(() => {});
				}
				term.dispose();
			}
			if (ptyId && isAlive && !existingPtyId) {
				killPty(ptyId);
			}
		};
	});
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="terminal-wrapper" onclick={handleContainerClick} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleContainerClick(); }} role="group" tabindex="-1">
	{#if spawnError}
		<div class="spawn-error">
			<span class="error-title">PTY spawn failed</span>
			<span class="error-detail">{spawnError}</span>
		</div>
	{/if}
	{#if showSearch}
		<div class="search-bar">
			<input
				bind:this={searchInput}
				bind:value={searchQuery}
				class="search-input"
				placeholder="Search..."
				onkeydown={handleSearchKeydown}
			/>
			<button class="search-btn" aria-label="Previous match" onclick={() => searchAddon?.findPrevious(searchQuery)}>Prev</button>
			<button class="search-btn" aria-label="Next match" onclick={() => searchAddon?.findNext(searchQuery)}>Next</button>
			<button class="search-btn" aria-label="Close search" onclick={() => (showSearch = false)}>Close</button>
		</div>
	{/if}
	<div bind:this={containerEl} class="terminal-container"></div>
</div>

<style>
	.terminal-wrapper {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		position: relative;
	}

	.terminal-container {
		width: 100%;
		flex: 1;
		min-height: 0;
	}

	.spawn-error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 8px;
		padding: 20px;
	}

	.error-title {
		color: var(--color-error, #ef4444);
		font-size: 14px;
		font-weight: 600;
	}

	.error-detail {
		color: var(--text-secondary);
		font-size: 12px;
		font-family: var(--font-code);
		max-width: 100%;
		word-break: break-all;
	}

	.search-bar {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		background: var(--bg-surface);
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.search-input {
		flex: 1;
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-button);
		color: var(--text-primary);
		font-size: 12px;
		padding: 3px 8px;
	}

	.search-input:focus {
		border-color: var(--accent);
	}

	.search-btn {
		background: none;
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		font-size: 11px;
		padding: 3px 8px;
		border-radius: var(--radius-button);
		cursor: pointer;
	}

	.search-btn:hover {
		color: var(--text-primary);
		border-color: var(--accent);
	}
</style>
