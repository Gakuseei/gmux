import { appStore } from '$lib/stores/app.svelte';
import { settingsStore } from '$lib/stores/settings.svelte';
import { splitNode, removeNode } from '$lib/utils/layout-helpers';
import type { SplitNode, TerminalSession } from '$lib/types/workspace';

function parseShortcut(shortcut: string): { ctrl: boolean; shift: boolean; alt: boolean; key: string } {
	const parts = shortcut.split('+');
	return {
		ctrl: parts.includes('Ctrl'),
		shift: parts.includes('Shift'),
		alt: parts.includes('Alt'),
		key: parts[parts.length - 1].toLowerCase(),
	};
}

function matchesShortcut(event: KeyboardEvent, shortcut: string): boolean {
	const parsed = parseShortcut(shortcut);
	const eventKey = event.key.toLowerCase();
	return (
		event.ctrlKey === parsed.ctrl &&
		event.shiftKey === parsed.shift &&
		event.altKey === parsed.alt &&
		eventKey === parsed.key.toLowerCase()
	);
}

function findFirstTerminalId(node: SplitNode): string | null {
	if (node.type === 'terminal') return node.terminalId ?? null;
	if (node.children) {
		for (const child of node.children) {
			const id = findFirstTerminalId(child);
			if (id) return id;
		}
	}
	return null;
}

function collectTerminalIds(node: SplitNode): string[] {
	if (node.type === 'terminal') return node.terminalId ? [node.terminalId] : [];
	const ids: string[] = [];
	if (node.children) {
		for (const child of node.children) {
			ids.push(...collectTerminalIds(child));
		}
	}
	return ids;
}

function splitActivePane(direction: 'horizontal' | 'vertical') {
	const ws = appStore.activeWorkspace;
	if (!ws) return;

	const targetId = appStore.activeTerminalId ?? findFirstTerminalId(ws.layout);
	if (!targetId) return;

	const newSession: TerminalSession = {
		id: crypto.randomUUID(),
		name: `Terminal ${ws.sessions.length + 1}`,
		shell: settingsStore.terminal.defaultShell,
		cwd: ws.cwd,
		status: 'running',
		notificationCount: 0,
	};

	ws.sessions.push(newSession);
	ws.layout = splitNode(ws.layout, targetId, direction, newSession);
	appStore.activeTerminalId = newSession.id;
}

function focusAdjacentPane(offset: number) {
	const ws = appStore.activeWorkspace;
	if (!ws) return;

	const ids = collectTerminalIds(ws.layout);
	if (ids.length === 0) return;

	const currentIdx = ids.indexOf(appStore.activeTerminalId ?? '');
	const nextIdx = (currentIdx + offset + ids.length) % ids.length;
	appStore.activeTerminalId = ids[nextIdx];
}

export function initKeybindings(): () => void {
	const handler = (e: KeyboardEvent) => {
		const target = e.target as HTMLElement;
		if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') {
			return;
		}

		const bindings = settingsStore.keybindings;

		const actions: Record<string, () => void> = {
			splitHorizontal: () => splitActivePane('horizontal'),
			splitVertical: () => splitActivePane('vertical'),
			closePane: () => {
				const ws = appStore.activeWorkspace;
				if (!ws || !appStore.activeTerminalId) return;
				const newLayout = removeNode(ws.layout, appStore.activeTerminalId);
				if (newLayout) {
					ws.sessions = ws.sessions.filter((s) => s.id !== appStore.activeTerminalId);
					ws.layout = newLayout;
					appStore.activeTerminalId = findFirstTerminalId(newLayout);
				}
			},
			newWorkspace: () => {
				appStore.showNewWorkspaceModal = true;
			},
			newTerminal: () => splitActivePane('horizontal'),
			toggleSidebar: () => {
				appStore.toggleSidebar();
			},
			search: () => {},
			nextPane: () => focusAdjacentPane(1),
			prevPane: () => focusAdjacentPane(-1),
		};

		for (const [action, shortcut] of Object.entries(bindings)) {
			if (matchesShortcut(e, shortcut)) {
				e.preventDefault();
				e.stopPropagation();
				actions[action]?.();
				return;
			}
		}
	};

	window.addEventListener('keydown', handler);
	return () => window.removeEventListener('keydown', handler);
}
