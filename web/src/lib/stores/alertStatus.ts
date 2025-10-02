import { writable, derived, get } from 'svelte/store';
import type { ThreatAlert, AlertStatus, AlertAcknowledgment } from '$lib/types';
import { browser } from '$app/environment';

interface AlertStatusState {
	acknowledgments: Record<string, AlertAcknowledgment>;
}

const STORAGE_KEY = 'ids_alert_statuses';

// Generate composite key: IP + threat type
function getStatusKey(alert: ThreatAlert): string {
	return `${alert.source_ip}-${alert.threat_type}`;
}

function loadFromStorage(): AlertStatusState {
	if (!browser) return { acknowledgments: {} };
	
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			return JSON.parse(stored);
		}
	} catch (err) {
		console.error('Failed to load alert statuses:', err);
	}
	return { acknowledgments: {} };
}

function saveToStorage(state: AlertStatusState) {
	if (!browser) return;
	
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
	} catch (err) {
		console.error('Failed to save alert statuses:', err);
	}
}

const createAlertStatusStore = () => {
	const { subscribe, update, set } = writable<AlertStatusState>(loadFromStorage());

	return {
		subscribe,
		setStatus: (alert: ThreatAlert, status: AlertStatus, notes?: string) => {
			update(state => {
				const key = getStatusKey(alert);
				const acknowledgment: AlertAcknowledgment = {
					alertId: alert.id,
					status,
					acknowledgedAt: new Date().toISOString(),
					notes
				};
				
				const newState = {
					...state,
					acknowledgments: {
						...state.acknowledgments,
						[key]: acknowledgment
					}
				};
				
				saveToStorage(newState);
				return newState;
			});
		},
		getStatus: (alert: ThreatAlert): AlertStatus => {
			const state = get({ subscribe });
			const key = getStatusKey(alert);
			return state.acknowledgments[key]?.status || 'new';
		},
		clearAll: () => {
			const newState = { acknowledgments: {} };
			saveToStorage(newState);
			set(newState);
		}
	};
};

export const alertStatusStore = createAlertStatusStore();