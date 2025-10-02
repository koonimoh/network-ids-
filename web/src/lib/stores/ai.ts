import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type AIProvider = 'openai' | 'anthropic';

interface AIConfig {
	provider: AIProvider;
	apiKey: string;
	model: string;
}

interface ChatMessage {
	role: 'user' | 'assistant';
	content: string;
	timestamp: Date;
}

const getStoredConfig = (): AIConfig => {
	if (!browser) return { provider: 'openai', apiKey: '', model: 'gpt-4o' };
	const stored = localStorage.getItem('ai_config');
	if (stored) {
		return JSON.parse(stored);
	}
	return { provider: 'openai', apiKey: '', model: 'gpt-4o' };
};

const createAIConfigStore = () => {
	const { subscribe, set, update } = writable<AIConfig>(getStoredConfig());

	return {
		subscribe,
		setProvider: (provider: AIProvider) => {
			update(config => {
				const newConfig = {
					...config,
					provider,
					model: provider === 'openai' ? 'gpt-4o' : 'claude-3-5-sonnet-20241022'
				};
				if (browser) localStorage.setItem('ai_config', JSON.stringify(newConfig));
				return newConfig;
			});
		},
		setApiKey: (apiKey: string) => {
			update(config => {
				const newConfig = { ...config, apiKey };
				if (browser) localStorage.setItem('ai_config', JSON.stringify(newConfig));
				return newConfig;
			});
		},
		setModel: (model: string) => {
			update(config => {
				const newConfig = { ...config, model };
				if (browser) localStorage.setItem('ai_config', JSON.stringify(newConfig));
				return newConfig;
			});
		},
		clear: () => {
			const newConfig = { provider: 'openai' as AIProvider, apiKey: '', model: 'gpt-4o' };
			if (browser) localStorage.removeItem('ai_config');
			set(newConfig);
		}
	};
};

const createChatHistoryStore = () => {
	const { subscribe, update, set } = writable<ChatMessage[]>([]);

	return {
		subscribe,
		addMessage: (message: ChatMessage) => {
			update(messages => [...messages, message]);
		},
		clear: () => set([])
	};
};

export const aiConfig = createAIConfigStore();
export const chatHistory = createChatHistoryStore();