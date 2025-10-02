import { writable } from 'svelte/store';
import type { AlertStatus } from '$lib/types';
import { browser } from '$app/environment';

export interface SavedFilter {
	id: string;
	name: string;
	description?: string;
	filters: {
		searchQuery: string;
		severityFilter: 'All' | 'Critical' | 'High' | 'Medium' | 'Low';
		statusFilter: 'all' | AlertStatus;
		sortOrder: 'newest' | 'oldest' | 'severity';
	};
	createdAt: string;
	color?: string;
}

const STORAGE_KEY = 'ids_saved_filters';

// Predefined default filters
const DEFAULT_FILTERS: SavedFilter[] = [
	{
		id: 'critical-unresolved',
		name: 'Critical Unresolved',
		description: 'Critical threats that need attention',
		filters: {
			searchQuery: '',
			severityFilter: 'Critical',
			statusFilter: 'new',
			sortOrder: 'newest'
		},
		createdAt: new Date().toISOString(),
		color: '#ef4444'
	},
	{
		id: 'high-investigating',
		name: 'Under Investigation',
		description: 'High severity alerts being investigated',
		filters: {
			searchQuery: '',
			severityFilter: 'High',
			statusFilter: 'investigating',
			sortOrder: 'newest'
		},
		createdAt: new Date().toISOString(),
		color: '#f97316'
	},
	{
		id: 'port-scans',
		name: 'Port Scans',
		description: 'All port scanning activity',
		filters: {
			searchQuery: 'port scan',
			severityFilter: 'All',
			statusFilter: 'all',
			sortOrder: 'newest'
		},
		createdAt: new Date().toISOString(),
		color: '#8b5cf6'
	},
	{
		id: 'ddos-attacks',
		name: 'DDoS Attacks',
		description: 'Potential DDoS attacks',
		filters: {
			searchQuery: 'ddos',
			severityFilter: 'All',
			statusFilter: 'all',
			sortOrder: 'severity'
		},
		createdAt: new Date().toISOString(),
		color: '#ec4899'
	}
];

function loadFilters(): SavedFilter[] {
	if (!browser) return DEFAULT_FILTERS;

	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const userFilters = JSON.parse(stored);
			// Combine defaults with user-created filters
			return [...DEFAULT_FILTERS, ...userFilters];
		}
	} catch (err) {
		console.error('Failed to load saved filters:', err);
	}

	return DEFAULT_FILTERS;
}

function saveFilters(filters: SavedFilter[]) {
	if (!browser) return;

	try {
		// Only save user-created filters (not defaults)
		const userFilters = filters.filter(f => !DEFAULT_FILTERS.some(df => df.id === f.id));
		localStorage.setItem(STORAGE_KEY, JSON.stringify(userFilters));
	} catch (err) {
		console.error('Failed to save filters:', err);
	}
}

const createSavedFiltersStore = () => {
	const { subscribe, update, set } = writable<SavedFilter[]>(loadFilters());

	return {
		subscribe,
		set,
		addFilter: (filter: Omit<SavedFilter, 'id' | 'createdAt'>) => {
			update(filters => {
				const newFilter: SavedFilter = {
					...filter,
					id: `custom-${Date.now()}`,
					createdAt: new Date().toISOString()
				};
				const updated = [...filters, newFilter];
				saveFilters(updated);
				return updated;
			});
		},
		updateFilter: (id: string, updates: Partial<SavedFilter>) => {
			update(filters => {
				const updated = filters.map(f => 
					f.id === id ? { ...f, ...updates } : f
				);
				saveFilters(updated);
				return updated;
			});
		},
		deleteFilter: (id: string) => {
			// Don't allow deleting default filters
			if (DEFAULT_FILTERS.some(df => df.id === id)) {
				return;
			}

			update(filters => {
				const updated = filters.filter(f => f.id !== id);
				saveFilters(updated);
				return updated;
			});
		},
		reset: () => {
			if (browser) {
				localStorage.removeItem(STORAGE_KEY);
			}
			set(DEFAULT_FILTERS);
		}
	};
};

export const savedFiltersStore = createSavedFiltersStore();