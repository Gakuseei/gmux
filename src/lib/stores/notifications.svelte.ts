import { appStore } from './app.svelte';
import { settingsStore } from './settings.svelte';

class NotificationStore {
	async notify(sessionId: string, patternName: string) {
		appStore.updateSessionStatus(sessionId, 'needs-input');
		appStore.incrementNotification(sessionId);

		if (settingsStore.notifications.desktopEnabled) {
			try {
				const { sendNotification } = await import('@tauri-apps/plugin-notification');
				sendNotification({ title: 'gmux', body: `Terminal needs input (${patternName})` });
			} catch (e) {
				console.error('Failed to send desktop notification:', e);
			}
		}
	}

	clear(sessionId: string) {
		appStore.clearNotification(sessionId);
	}
}

export const notifications = new NotificationStore();
