<script lang="ts">
	import { Terminal } from '@xterm/xterm';
	import { FitAddon } from '@xterm/addon-fit';
	import { WebglAddon } from '@xterm/addon-webgl';
	import { WebLinksAddon } from '@xterm/addon-web-links';
	import '@xterm/xterm/css/xterm.css';
	import { createPty, writePty, resizePty, killPty } from './terminal-bridge';
	import { appStore } from '$lib/stores/app.svelte';
	import { notifications } from '$lib/stores/notifications.svelte';
	import { detectNotification, createLineBuffer } from '$lib/utils/notification-detector';

	let {
		terminalId,
		shell = '/bin/bash',
		cwd,
		command,
		onTitleChange,
		onData
	}: {
		terminalId: string;
		shell?: string;
		cwd: string;
		command?: string;
		onTitleChange?: (title: string) => void;
		onData?: (data: string) => void;
	} = $props();

	const lineBuffer = createLineBuffer((line) => {
		const result = detectNotification(line);
		if (result.matched && appStore.activeTerminalId !== terminalId) {
			notifications.notify(terminalId, result.pattern);
		}
	});

	function handleContainerClick() {
		appStore.activeTerminalId = terminalId;
		notifications.clear(terminalId);
	}

	let containerEl: HTMLDivElement | undefined = $state();
	let ptyId: string | null = $state(null);
	let isAlive: boolean = $state(false);

	$effect(() => {
		if (!containerEl) return;

		let term: Terminal | null = null;
		let fitAddon: FitAddon | null = null;
		let resizeObserver: ResizeObserver | null = null;
		let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
		let disposed = false;

		const setup = async () => {
			term = new Terminal({
				fontFamily: 'JetBrains Mono, monospace',
				fontSize: 14,
				theme: {
					background: '#171717',
					foreground: '#e5e5e5',
					cursor: '#10a37f',
					selectionBackground: 'rgba(16, 163, 127, 0.3)'
				},
				cursorBlink: true,
				allowProposedApi: true
			});

			fitAddon = new FitAddon();
			term.loadAddon(fitAddon);
			term.loadAddon(new WebLinksAddon());

			term.open(containerEl!);

			try {
				term.loadAddon(new WebglAddon());
			} catch {
			}

			fitAddon.fit();

			if (onTitleChange) {
				term.onTitleChange(onTitleChange);
			}

			const decoder = new TextDecoder();

			const id = await createPty(
				shell,
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
				},
				(code) => {
					isAlive = false;
				}
			);

			if (disposed) {
				killPty(id);
				return;
			}

			ptyId = id;
			isAlive = true;

			term.onData((input) => {
				if (ptyId && isAlive) {
					writePty(ptyId, input);
				}
			});

			if (command) {
				setTimeout(() => {
					if (ptyId && isAlive) {
						writePty(ptyId, command + '\r');
					}
				}, 200);
			}

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
			if (term) term.dispose();
			if (ptyId && isAlive) {
				killPty(ptyId);
			}
		};
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div bind:this={containerEl} class="terminal-container" onclick={handleContainerClick}></div>

<style>
	.terminal-container {
		width: 100%;
		height: 100%;
	}
</style>
