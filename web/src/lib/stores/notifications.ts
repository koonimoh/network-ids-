import { writable } from 'svelte/store';
import type { ThreatAlert } from '$lib/types';
import { browser } from '$app/environment';

interface NotificationSettings {
	enabled: boolean;
	soundEnabled: boolean;
	minSeverity: 'Low' | 'Medium' | 'High' | 'Critical';
	permission: NotificationPermission;
}

const STORAGE_KEY = 'ids_notification_settings';

function loadSettings(): NotificationSettings {
	if (!browser) {
		return {
			enabled: false,
			soundEnabled: true,
			minSeverity: 'High',
			permission: 'default'
		};
	}

	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const settings = JSON.parse(stored);
			// Update permission status from browser
			settings.permission = Notification.permission;
			return settings;
		}
	} catch (err) {
		console.error('Failed to load notification settings:', err);
	}

	return {
		enabled: false,
		soundEnabled: true,
		minSeverity: 'High',
		permission: Notification.permission
	};
}

function saveSettings(settings: NotificationSettings) {
	if (!browser) return;
	
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
	} catch (err) {
		console.error('Failed to save notification settings:', err);
	}
}

const createNotificationStore = () => {
	const { subscribe, update, set } = writable<NotificationSettings>(loadSettings());

	// Audio for notification sound
	let audioContext: AudioContext | null = null;
	
	const playNotificationSound = () => {
		if (!browser) return;
		
		try {
			if (!audioContext) {
				audioContext = new AudioContext();
			}

			// Create a simple beep sound
			const oscillator = audioContext.createOscillator();
			const gainNode = audioContext.createGain();

			oscillator.connect(gainNode);
			gainNode.connect(audioContext.destination);

			oscillator.frequency.value = 800; // Hz
			oscillator.type = 'sine';

			gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
			gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);

			oscillator.start(audioContext.currentTime);
			oscillator.stop(audioContext.currentTime + 0.5);
		} catch (err) {
			console.error('Failed to play notification sound:', err);
		}
	};

	const requestPermission = async () => {
		if (!browser || !('Notification' in window)) {
			return false;
		}

		try {
			const permission = await Notification.requestPermission();
			update(state => {
				const newState = { ...state, permission };
				saveSettings(newState);
				return newState;
			});
			return permission === 'granted';
		} catch (err) {
			console.error('Failed to request notification permission:', err);
			return false;
		}
	};

	const shouldNotify = (settings: NotificationSettings, severity: string): boolean => {
		if (!settings.enabled || settings.permission !== 'granted') {
			return false;
		}

		const severityOrder = { Low: 1, Medium: 2, High: 3, Critical: 4 };
		const alertLevel = severityOrder[severity as keyof typeof severityOrder] || 0;
		const minLevel = severityOrder[settings.minSeverity];

		return alertLevel >= minLevel;
	};

	const sendNotification = (alert: ThreatAlert, settings: NotificationSettings) => {
		if (!browser || !('Notification' in window)) return;

		if (!shouldNotify(settings, alert.severity)) {
			return;
		}

		try {
			const notification = new Notification('🚨 Network Threat Detected', {
				body: `${alert.severity.toUpperCase()}: ${alert.threat_type}\nSource: ${alert.source_ip}\n${alert.description}`,
				icon: '/favicon.png',
				badge: '/favicon.png',
				tag: alert.id,
				requireInteraction: alert.severity === 'Critical',
				silent: !settings.soundEnabled
			});

			// Play sound if enabled
			if (settings.soundEnabled && (alert.severity === 'Critical' || alert.severity === 'High')) {
				playNotificationSound();
			}

			// Auto-close after 10 seconds for non-critical
			if (alert.severity !== 'Critical') {
				setTimeout(() => notification.close(), 10000);
			}
		} catch (err) {
			console.error('Failed to send notification:', err);
		}
	};

	return {
		subscribe,
		set,
		requestPermission,
		sendNotification: (alert: ThreatAlert) => {
			let currentSettings: NotificationSettings;
			subscribe(s => currentSettings = s)();
			sendNotification(alert, currentSettings);
		},
		toggleEnabled: () => {
			update(state => {
				const newState = { ...state, enabled: !state.enabled };
				saveSettings(newState);
				return newState;
			});
		},
		toggleSound: () => {
			update(state => {
				const newState = { ...state, soundEnabled: !state.soundEnabled };
				saveSettings(newState);
				return newState;
			});
		},
		setMinSeverity: (severity: NotificationSettings['minSeverity']) => {
			update(state => {
				const newState = { ...state, minSeverity: severity };
				saveSettings(newState);
				return newState;
			});
		}
	};
};

export const notificationStore = createNotificationStore();