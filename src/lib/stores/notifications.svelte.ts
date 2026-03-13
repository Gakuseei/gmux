import { appStore } from './app.svelte';

class NotificationStore {
	private desktopEnabled = $state(true);

	async notify(sessionId: string, patternName: string) {
		appStore.updateSessionStatus(sessionId, 'needs-input');
		appStore.incrementNotification(sessionId);

		if (this.desktopEnabled) {
			try {
				const { sendNotification } = await import('@tauri-apps/plugin-notification');
				sendNotification({ title: 'gmux', body: `Terminal needs input (${patternName})` });
			} catch {
			}
		}
	}

	clear(sessionId: string) {
		appStore.clearNotification(sessionId);
	}

	setDesktopEnabled(enabled: boolean) {
		this.desktopEnabled = enabled;
	}
}

export const notifications = new NotificationStore();
