import { invoke } from '@tauri-apps/api/core';

class SettingsStore {
	appearance = $state({
		accentColor: '#10a37f',
		fontUi: 'Inter, system-ui, sans-serif',
		fontCode: '"JetBrains Mono", monospace',
		fontSize: 14,
	});
	terminal = $state({
		defaultShell: '/bin/bash',
		scrollbackLines: 10000,
		cursorStyle: 'block' as 'block' | 'beam' | 'underline',
	});
	aiClis = $state({
		claude: { path: 'claude', enabled: true },
		codex: { path: 'codex', enabled: true },
		gemini: { path: 'gemini', enabled: true },
		custom: [] as Array<{ name: string; command: string }>,
	});
	rateLimits = $state({
		claude: { fiveHourLimit: 1000000, weeklyLimit: 5000000, resetDay: 'monday', resetHour: 0 },
		codex: { fiveHourLimit: 0, weeklyLimit: 0, resetDay: 'monday', resetHour: 0 },
		gemini: { fiveHourLimit: 0, weeklyLimit: 0, resetDay: 'monday', resetHour: 0 },
	});
	notifications = $state({
		desktopEnabled: true,
		soundEnabled: false,
		customPatterns: [] as string[],
	});
	keybindings = $state({
		splitHorizontal: 'Ctrl+Shift+D',
		splitVertical: 'Ctrl+Shift+R',
		closePane: 'Ctrl+Shift+W',
		newWorkspace: 'Ctrl+Shift+N',
		newTerminal: 'Ctrl+Shift+T',
		toggleSidebar: 'Ctrl+B',
		search: 'Ctrl+Shift+F',
		nextPane: 'Ctrl+Tab',
		prevPane: 'Ctrl+Shift+Tab',
	});

	async load() {
		try {
			const raw = await invoke<string | null>('load_settings');
			if (raw) {
				const parsed = JSON.parse(raw);
				if (parsed.appearance) Object.assign(this.appearance, parsed.appearance);
				if (parsed.terminal) Object.assign(this.terminal, parsed.terminal);
				if (parsed.aiClis) Object.assign(this.aiClis, parsed.aiClis);
				if (parsed.rateLimits) Object.assign(this.rateLimits, parsed.rateLimits);
				if (parsed.notifications) Object.assign(this.notifications, parsed.notifications);
				if (parsed.keybindings) Object.assign(this.keybindings, parsed.keybindings);
			}
		} catch {
		}
	}

	async save() {
		const data = JSON.stringify({
			appearance: { ...this.appearance },
			terminal: { ...this.terminal },
			aiClis: { ...this.aiClis },
			rateLimits: { ...this.rateLimits },
			notifications: { ...this.notifications },
			keybindings: { ...this.keybindings },
		});
		try {
			await invoke('save_settings', { data });
		} catch {
		}
	}

	applyAppearance() {
		const root = document.documentElement;
		root.style.setProperty('--accent', this.appearance.accentColor);
		root.style.setProperty('--font-ui', this.appearance.fontUi);
		root.style.setProperty('--font-code', this.appearance.fontCode);
		root.style.setProperty('--font-size', `${this.appearance.fontSize}px`);
		root.style.fontFamily = this.appearance.fontUi;
	}
}

export const settingsStore = new SettingsStore();
