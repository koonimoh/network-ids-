import { writable } from 'svelte/store';

export type Page = 'dashboard' | 'analytics' | 'threat-map' | 'ai-assistant' | 'settings';

export const currentPage = writable<Page>('dashboard');