import { invoke } from '@tauri-apps/api/core';

interface SessionUsage {
	session_id: string;
	input_tokens: number;
	output_tokens: number;
	cache_read_tokens: number;
	cache_write_tokens: number;
	timestamp: string;
}

interface UsageData {
	total_input: number;
	total_output: number;
	total_cache_read: number;
	total_cache_write: number;
	sessions: SessionUsage[];
}

class UsageStore {
	period = $state<'today' | 'weekly' | 'monthly'>('today');
	data = $state<UsageData | null>(null);
	loading = $state(false);

	async refresh() {
		this.loading = true;
		try {
			this.data = await invoke<UsageData>('get_usage_data', { period: this.period });
		} catch (e) {
			console.error('Failed to load usage data:', e);
			this.data = null;
		}
		this.loading = false;
	}

	setPeriod(period: 'today' | 'weekly' | 'monthly') {
		this.period = period;
		this.refresh();
	}
}

export const usageStore = new UsageStore();
