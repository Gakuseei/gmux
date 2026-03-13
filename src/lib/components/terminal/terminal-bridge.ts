import { invoke, Channel } from '@tauri-apps/api/core';

type TerminalEvent =
	| { event: 'output'; data: { data: number[] } }
	| { event: 'exit'; data: { code: number | null } };

export async function createPty(
	shell: string,
	cwd: string,
	cols: number,
	rows: number,
	onData: (data: Uint8Array) => void,
	onExit: (code: number | null) => void
): Promise<string> {
	const channel = new Channel<TerminalEvent>();

	channel.onmessage = (msg) => {
		if (msg.event === 'output') {
			onData(new Uint8Array(msg.data.data));
		} else if (msg.event === 'exit') {
			onExit(msg.data.code);
		}
	};

	return await invoke<string>('create_pty', {
		shell,
		cwd,
		cols,
		rows,
		onEvent: channel
	});
}

export async function writePty(id: string, data: string): Promise<void> {
	await invoke('write_pty', { id, data });
}

export async function resizePty(id: string, rows: number, cols: number): Promise<void> {
	await invoke('resize_pty', { id, rows, cols });
}

export async function killPty(id: string): Promise<void> {
	await invoke('kill_pty', { id });
}

type BatchTerminalEvent = {
	index: number;
	event: TerminalEvent;
};

export async function spawnBatch(
	requests: Array<{
		shell: string;
		cwd: string;
		command?: string;
		cols: number;
		rows: number;
	}>,
	callbacks: Array<{
		onData: (data: Uint8Array) => void;
		onExit: (code: number | null) => void;
	}>
): Promise<string[]> {
	const channel = new Channel<BatchTerminalEvent>();

	channel.onmessage = (msg) => {
		const cb = callbacks[msg.index];
		if (!cb) return;
		if (msg.event.event === 'output') {
			cb.onData(new Uint8Array(msg.event.data.data));
		} else if (msg.event.event === 'exit') {
			cb.onExit(msg.event.data.code);
		}
	};

	return await invoke<string[]>('spawn_batch', {
		requests,
		onEvent: channel
	});
}
