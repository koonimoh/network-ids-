import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

const getInitialTheme = (): Theme => {
	if (!browser) return 'dark';
	const stored = localStorage.getItem('theme') as Theme;
	if (stored) return stored;
	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

const createThemeStore = () => {
	const { subscribe, set } = writable<Theme>(getInitialTheme());

	return {
		subscribe,
		toggle: () => {
			if (!browser) return;
			const current = document.documentElement.getAttribute('data-theme') as Theme;
			const next: Theme = current === 'dark' ? 'light' : 'dark';
			document.documentElement.setAttribute('data-theme', next);
			localStorage.setItem('theme', next);
			set(next);
		},
		init: () => {
			if (!browser) return;
			const theme = getInitialTheme();
			document.documentElement.setAttribute('data-theme', theme);
			set(theme);
		}
	};
};

export const theme = createThemeStore();