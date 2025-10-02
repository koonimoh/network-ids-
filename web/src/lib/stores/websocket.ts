import { writable } from 'svelte/store';
import type { ThreatAlert, ApiResponse } from '$lib/types';
import { browser } from '$app/environment';
import { notificationStore } from './notifications';

interface WebSocketState {
	connected: boolean;
	alerts: ThreatAlert[];
	error: string | null;
}

const createWebSocketStore = () => {
	const { subscribe, update } = writable<WebSocketState>({
		connected: false,
		alerts: [],
		error: null
	});

	let ws: WebSocket | null = null;
	let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

	const connect = () => {
		console.log('🔵 connect() called');
		if (!browser){ 
		console.log('🔴 Not in browser, aborting');
		return;
		}
		
		try {
			const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
			const host = window.location.hostname;
			const port = import.meta.env.DEV ? '3000' : window.location.port;
			
			console.log('🟢 Creating WebSocket to ws://localhost:3000/ws/alerts');
			ws = new WebSocket('ws://localhost:3000/ws/alerts');

			ws.onopen = () => {
				console.log('✅ WebSocket connected');
				update(state => ({ ...state, connected: true, error: null }));
				console.log('WebSocket connected');
			};

			ws.onmessage = (event) => {
				try {
					const response: ApiResponse<ThreatAlert> = JSON.parse(event.data);
					if (response.success && response.data) {
						const alert = response.data;
						update(state => ({
							...state,
							alerts: [alert, ...state.alerts].slice(0, 100)
						}));
						// Send browser notification
						notificationStore.sendNotification(alert);
					}
				} catch (err) {
					console.error('Failed to parse WebSocket message:', err);
				}
			};

			ws.onerror = (error) => {
				console.error('❌ WebSocket error:', error);
				update(state => ({ ...state, error: 'Connection error' }));
			};

			ws.onclose = () => {
				update(state => ({ ...state, connected: false }));
				console.log('WebSocket disconnected');
				
				// Attempt reconnection after 3 seconds
				if (reconnectTimeout) clearTimeout(reconnectTimeout);
				reconnectTimeout = setTimeout(() => {
					console.log('Attempting to reconnect...');
					connect();
				}, 3000);
			};
		} catch (err) {
			console.error('Failed to create WebSocket:', err);
			update(state => ({ ...state, error: 'Failed to connect' }));
		}
	};

	const disconnect = () => {
		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
			reconnectTimeout = null;
		}
		if (ws) {
			ws.close();
			ws = null;
		}
		update(state => ({ ...state, connected: false }));
	};

	return {
		subscribe,
		connect,
		disconnect,
		clearAlerts: () => update(state => ({ ...state, alerts: [] }))
	};
};

export const wsStore = createWebSocketStore();