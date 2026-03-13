import { sendNotification } from '@tauri-apps/plugin-notification';
import { appStore } from './app.svelte';
import { settingsStore } from './settings.svelte';

const BEEP_FREQUENCY = 880;
const BEEP_DURATION = 0.15;
const BEEP_VOLUME = 0.3;

class NotificationStore {
	private audioCtx: AudioContext | null = null;

	private playBeep() {
		if (!this.audioCtx) {
			this.audioCtx = new AudioContext();
		}
		const ctx = this.audioCtx;
		const oscillator = ctx.createOscillator();
		const gain = ctx.createGain();
		oscillator.type = 'sine';
		oscillator.frequency.setValueAtTime(BEEP_FREQUENCY, ctx.currentTime);
		gain.gain.setValueAtTime(BEEP_VOLUME, ctx.currentTime);
		gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + BEEP_DURATION);
		oscillator.connect(gain);
		gain.connect(ctx.destination);
		oscillator.start(ctx.currentTime);
		oscillator.stop(ctx.currentTime + BEEP_DURATION);
		oscillator.onended = () => { gain.disconnect(); oscillator.disconnect(); };
	}

	async notify(sessionId: string, patternName: string) {
		appStore.updateSessionStatus(sessionId, 'needs-input');
		appStore.incrementNotification(sessionId);

		if (settingsStore.notifications.soundEnabled) {
			this.playBeep();
		}

		if (settingsStore.notifications.desktopEnabled) {
			try {
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
