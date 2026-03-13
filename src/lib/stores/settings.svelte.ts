import { invoke } from '@tauri-apps/api/core';

class SettingsStore {
	appearance = $state({
		accentColor: '#10a37f',
		fontUi: 'Inter, system-ui, sans-serif',
		fontCode: '"JetBrains Mono", monospace',
		fontSize: 14,
	});
	terminal = $state({
		defaultShell: '',
		scrollbackLines: 10000,
		cursorStyle: 'block' as 'block' | 'bar' | 'underline',
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
	costRates = $state({
		claude: { input: 3, output: 15, cacheRead: 0.3, cacheWrite: 3.75 },
		codex: { input: 2.5, output: 10, cacheRead: 0.25, cacheWrite: 3 },
		gemini: { input: 1.25, output: 5, cacheRead: 0.3, cacheWrite: 1.25 },
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

	private validateAndAssign<T extends Record<string, unknown>>(target: T, source: unknown): void {
		if (!source || typeof source !== 'object') return;
		const src = source as Record<string, unknown>;
		for (const key of Object.keys(target)) {
			if (!(key in src)) continue;
			const defaultVal = target[key];
			const newVal = src[key];
			if (typeof defaultVal === 'number') {
				if (typeof newVal === 'number' && Number.isFinite(newVal) && newVal >= 0) {
					(target as Record<string, unknown>)[key] = newVal;
				}
			} else if (typeof defaultVal === 'boolean') {
				if (typeof newVal === 'boolean') {
					(target as Record<string, unknown>)[key] = newVal;
				}
			} else if (typeof defaultVal === 'string') {
				if (typeof newVal === 'string') {
					(target as Record<string, unknown>)[key] = newVal;
				}
			} else if (Array.isArray(defaultVal)) {
				if (Array.isArray(newVal)) {
					(target as Record<string, unknown>)[key] = newVal;
				}
			} else if (typeof defaultVal === 'object' && defaultVal !== null) {
				if (typeof newVal === 'object' && newVal !== null && !Array.isArray(newVal)) {
					(target as Record<string, unknown>)[key] = newVal;
				}
			}
		}
	}

	async load() {
		try {
			const raw = await invoke<string | null>('load_settings');
			if (raw) {
				const parsed = JSON.parse(raw);
				if (parsed.appearance) this.validateAndAssign(this.appearance, parsed.appearance);
				if (parsed.terminal) this.validateAndAssign(this.terminal, parsed.terminal);
				if (parsed.aiClis) this.validateAndAssign(this.aiClis, parsed.aiClis);
				if (parsed.rateLimits) this.validateAndAssign(this.rateLimits, parsed.rateLimits);
				if (parsed.notifications) this.validateAndAssign(this.notifications, parsed.notifications);
				if (parsed.costRates) this.validateAndAssign(this.costRates, parsed.costRates);
				if (parsed.keybindings) this.validateAndAssign(this.keybindings, parsed.keybindings);
			}
		} catch (e) {
			console.error('Failed to load settings:', e);
		}

		if (!this.terminal.defaultShell) {
			try {
				this.terminal.defaultShell = await invoke<string>('get_default_shell');
			} catch (e) {
				console.error('Failed to detect default shell:', e);
				this.terminal.defaultShell = '/bin/bash';
			}
		}
	}

	async save() {
		const data = JSON.stringify({
			appearance: { ...this.appearance },
			terminal: { ...this.terminal },
			aiClis: { ...this.aiClis },
			rateLimits: { ...this.rateLimits },
			notifications: { ...this.notifications },
			costRates: { ...this.costRates },
			keybindings: { ...this.keybindings },
		});
		try {
			await invoke('save_settings', { data });
		} catch (e) {
			console.error('Failed to save settings:', e);
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
