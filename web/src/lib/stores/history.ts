import { writable } from 'svelte/store';
import type { SystemStats } from '$lib/types';

interface HistoryDataPoint {
	timestamp: number;
	threats: number;
	packets: number;
	bandwidth: number; // bytes per second
}

interface HistoryState {
	data: HistoryDataPoint[];
	maxDataPoints: number;
}

const MAX_HISTORY_POINTS = 60; // Keep last 60 data points (1 minute at 1sec intervals)

const createHistoryStore = () => {
	const { subscribe, update } = writable<HistoryState>({
		data: [],
		maxDataPoints: MAX_HISTORY_POINTS
	});

	let lastStats: SystemStats | null = null;

	return {
		subscribe,
		addDataPoint: (stats: SystemStats) => {
			const now = Date.now();
			
			// Calculate bandwidth (bytes processed since last update)
			let bandwidth = 0;
			if (lastStats) {
				const bytesDiff = stats.bytes_processed - lastStats.bytes_processed;
				bandwidth = bytesDiff; // bytes per second (since we update every second)
			}
			
			lastStats = stats;

			update(state => {
				const newPoint: HistoryDataPoint = {
					timestamp: now,
					threats: stats.threats_detected,
					packets: stats.packets_processed,
					bandwidth
				};

				const newData = [...state.data, newPoint];
				
				// Keep only the last N points
				if (newData.length > state.maxDataPoints) {
					newData.shift();
				}

				return { ...state, data: newData };
			});
		},
		clear: () => {
			lastStats = null;
			update(state => ({ ...state, data: [] }));
		},
		setMaxPoints: (max: number) => {
			update(state => {
				const newData = state.data.slice(-max);
				return { ...state, data: newData, maxDataPoints: max };
			});
		}
	};
};

export const historyStore = createHistoryStore();