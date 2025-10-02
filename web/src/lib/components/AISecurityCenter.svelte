<script lang="ts">
	import { onMount } from 'svelte';
	import type { AIQueryRequest, AIQueryResponse, ChatMessage } from '$lib/types';

	type AIProvider = 'openai' | 'anthropic' | 'gemini';

	let selectedProvider: AIProvider = 'openai';
	let userQuery = '';
	let isLoading = false;
	let error = '';
	let chatHistory: Array<ChatMessage & { timestamp: Date }> = [];
	let apiKeyConfigured = {
		openai: false,
		anthropic: false,
		gemini: false
	};

	const providers = [
		{
			id: 'openai' as AIProvider,
			name: 'OpenAI',
			model: 'GPT-4o',
			description: 'Most versatile, great for general analysis'
		},
		{
			id: 'anthropic' as AIProvider,
			name: 'Anthropic',
			model: 'Claude Sonnet 4',
			description: 'Best for detailed threat analysis'
		},
		{
			id: 'gemini' as AIProvider,
			name: 'Google',
			model: 'Gemini 2.5 Flash',
			description: 'Fast and cost-effective'
		}
	];

	const quickActions = [
		{
			icon: '🎯',
			label: 'Top Threats',
			prompt: 'What are the 3 most critical threats I should address right now?'
		},
		{
			icon: '📊',
			label: 'Security Report',
			prompt: 'Generate a concise security summary of the current system status'
		},
		{
			icon: '🔍',
			label: 'Anomaly Check',
			prompt: 'Are there any unusual patterns or anomalies in the recent traffic?'
		},
		{
			icon: '✅',
			label: 'Action Items',
			prompt: 'Give me a prioritized list of actions I should take based on current alerts'
		}
	];

	async function sendQuery(query: string) {
		if (!query.trim() || isLoading) return;

		const message = query.trim();
		userQuery = '';
		error = '';
		isLoading = true;

		// Add user message
		chatHistory = [
			...chatHistory,
			{ role: 'user', content: message, timestamp: new Date() }
		];

		try {
			const request: AIQueryRequest = {
				query: message,
				provider: selectedProvider,
				conversation_history: chatHistory.slice(0, -1).map(({ role, content }) => ({ role, content }))
			};

			const res = await fetch('/api/ai/query', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(request)
			});

			const data = await res.json();

			if (data.success && data.data) {
				const response: AIQueryResponse = data.data;
				chatHistory = [
					...chatHistory,
					{ role: 'assistant', content: response.response, timestamp: new Date() }
				];
			} else {
				error = data.error || 'Failed to get AI response';
			}
		} catch (err: any) {
			console.error('AI query failed:', err);
			error = err.message || 'Failed to connect to AI service';
		} finally {
			isLoading = false;
		}
	}

	function handleKeyPress(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			sendQuery(userQuery);
		}
	}

	function clearChat() {
		if (confirm('Clear chat history?')) {
			chatHistory = [];
			error = '';
		}
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
	}
</script>

<div class="ai-security-center">
	<div class="page-header">
		<div>
			<h1>AI Security Operations Center</h1>
			<p class="subtitle">Natural language threat analysis powered by advanced AI</p>
		</div>
	</div>

	<!-- Provider Selection -->
	<div class="provider-selection card">
		<h3>Select AI Provider</h3>
		<div class="providers">
			{#each providers as provider}
				<button
					class="provider-card"
					class:active={selectedProvider === provider.id}
					on:click={() => (selectedProvider = provider.id)}
				>
					<div class="provider-header">
						<span class="provider-name">{provider.name}</span>
						<span class="provider-model mono">{provider.model}</span>
					</div>
					<p class="provider-description">{provider.description}</p>
				</button>
			{/each}
		</div>
	</div>

	<!-- Quick Actions -->
	<div class="quick-actions card">
		<h3>Quick Actions</h3>
		<div class="actions-grid">
			{#each quickActions as action}
				<button class="action-btn" on:click={() => sendQuery(action.prompt)} disabled={isLoading}>
					<span class="action-icon">{action.icon}</span>
					<span class="action-label">{action.label}</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Chat Interface -->
	<div class="chat-container card">
		<div class="chat-header">
			<h3>AI Analyst Chat</h3>
			{#if chatHistory.length > 0}
				<button class="btn-clear" on:click={clearChat}>Clear History</button>
			{/if}
		</div>

		<div class="chat-messages">
			{#if chatHistory.length === 0}
				<div class="empty-state">
					<svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
					</svg>
					<h4>Ask Your AI Security Analyst</h4>
					<p>Use natural language to query your IDS data</p>
					<ul>
						<li>"Show me all failed SSH attempts from China"</li>
						<li>"What's the most suspicious IP right now?"</li>
						<li>"Find unusual outbound connections"</li>
					</ul>
				</div>
			{:else}
				{#each chatHistory as message}
					<div class="message" class:user={message.role === 'user'}>
						<div class="message-header">
							<span class="message-role">
								{message.role === 'user' ? '👤 You' : '🤖 AI Analyst'}
							</span>
							<span class="message-time mono">
								{message.timestamp.toLocaleTimeString()}
							</span>
						</div>
						<div class="message-content">{message.content}</div>
						<button
							class="copy-btn"
							on:click={() => copyToClipboard(message.content)}
							title="Copy"
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
							</svg>
						</button>
					</div>
				{/each}

				{#if isLoading}
					<div class="message assistant loading">
						<div class="message-header">
							<span class="message-role">🤖 AI Analyst</span>
						</div>
						<div class="message-content">
							<div class="typing-indicator">
								<span></span><span></span><span></span>
							</div>
						</div>
					</div>
				{/if}
			{/if}
		</div>

		{#if error}
			<div class="error-banner">
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<circle cx="12" cy="12" r="10" />
					<line x1="12" y1="8" x2="12" y2="12" />
					<line x1="12" y1="16" x2="12.01" y2="16" />
				</svg>
				{error}
			</div>
		{/if}

		<div class="chat-input">
			<textarea
				bind:value={userQuery}
				placeholder="Ask anything about your network security..."
				on:keypress={handleKeyPress}
				disabled={isLoading}
				rows="3"
			></textarea>
			<button class="send-btn" on:click={() => sendQuery(userQuery)} disabled={!userQuery.trim() || isLoading}>
				{#if isLoading}
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
						<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2" />
					</svg>
				{:else}
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<line x1="22" y1="2" x2="11" y2="13" />
						<polygon points="22 2 15 22 11 13 2 9 22 2" />
					</svg>
				{/if}
			</button>
		</div>
	</div>
</div>

<style>
	.ai-security-center {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.page-header h1 {
		font-size: 1.875rem;
		font-weight: 700;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		color: var(--text-secondary);
		font-size: 0.875rem;
	}

	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.card h3 {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 1rem;
	}

	/* Provider Selection */
	.providers {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 1rem;
	}

	.provider-card {
		padding: 1rem;
		background: var(--bg-tertiary);
		border: 2px solid var(--border-primary);
		border-radius: 0.5rem;
		text-align: left;
		transition: all 0.2s;
		cursor: pointer;
	}

	.provider-card:hover {
		border-color: var(--border-secondary);
		background: var(--bg-hover);
	}

	.provider-card.active {
		border-color: var(--accent-primary);
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
	}

	.provider-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.provider-name {
		font-size: 0.9375rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.provider-model {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.provider-description {
		font-size: 0.8125rem;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	/* Quick Actions */
	.actions-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 0.75rem;
	}

	.action-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		padding: 1rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		transition: all 0.2s;
		cursor: pointer;
	}

	.action-btn:hover:not(:disabled) {
		background: var(--bg-hover);
		border-color: var(--accent-primary);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.action-icon {
		font-size: 1.5rem;
	}

	.action-label {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-primary);
		text-align: center;
	}

	/* Chat */
	.chat-container {
		display: flex;
		flex-direction: column;
		height: 600px;
	}

	.chat-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.btn-clear {
		padding: 0.5rem 1rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.375rem;
		font-size: 0.875rem;
		transition: all 0.2s;
	}

	.btn-clear:hover {
		background: var(--bg-hover);
	}

	.chat-messages {
		flex: 1;
		overflow-y: auto;
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
		margin-bottom: 1rem;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		padding: 3rem 1rem;
		color: var(--text-secondary);
	}

	.empty-state svg {
		margin-bottom: 1rem;
		opacity: 0.5;
	}

	.empty-state h4 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
		color: var(--text-primary);
	}

	.empty-state ul {
		list-style: none;
		text-align: left;
		margin-top: 1rem;
	}

	.empty-state li {
		padding: 0.5rem 0;
		padding-left: 1.5rem;
		position: relative;
	}

	.empty-state li::before {
		content: '→';
		position: absolute;
		left: 0;
		color: var(--accent-primary);
	}

	.message {
		padding: 1rem;
		background: var(--bg-secondary);
		border-radius: 0.75rem;
		position: relative;
	}

	.message.user {
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
		margin-left: 2rem;
	}

	.message-header {
		display: flex;
		justify-content: space-between;
		margin-bottom: 0.5rem;
	}

	.message-role {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.message-time {
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	.message-content {
		font-size: 0.875rem;
		line-height: 1.6;
		color: var(--text-primary);
		white-space: pre-wrap;
	}

	.copy-btn {
		position: absolute;
		top: 0.75rem;
		right: 0.75rem;
		width: 1.75rem;
		height: 1.75rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.375rem;
		color: var(--text-secondary);
		opacity: 0;
		transition: all 0.2s;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.message:hover .copy-btn {
		opacity: 1;
	}

	.copy-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.typing-indicator {
		display: flex;
		gap: 0.5rem;
	}

	.typing-indicator span {
		width: 0.5rem;
		height: 0.5rem;
		background: var(--text-secondary);
		border-radius: 50%;
		animation: typing 1.4s infinite;
	}

	.typing-indicator span:nth-child(2) {
		animation-delay: 0.2s;
	}

	.typing-indicator span:nth-child(3) {
		animation-delay: 0.4s;
	}

	@keyframes typing {
		0%,
		60%,
		100% {
			opacity: 0.3;
			transform: translateY(0);
		}
		30% {
			opacity: 1;
			transform: translateY(-0.5rem);
		}
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.875rem 1rem;
		background: color-mix(in srgb, var(--error) 10%, transparent);
		border: 1px solid var(--error);
		border-radius: 0.5rem;
		color: var(--error);
		font-size: 0.875rem;
		margin-bottom: 1rem;
	}

	.chat-input {
		position: relative;
		display: flex;
		gap: 0.75rem;
	}

	textarea {
		flex: 1;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-family: inherit;
		color: var(--text-primary);
		resize: vertical;
		min-height: 80px;
	}

	textarea:focus {
		outline: none;
		border-color: var(--accent-primary);
	}

	textarea:disabled {
		opacity: 0.5;
	}

	.send-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 3rem;
		background: var(--accent-primary);
		color: var(--text-inverse);
		border-radius: 0.5rem;
		transition: all 0.2s;
	}

	.send-btn:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.send-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	@media (max-width: 1024px) {
		.providers {
			grid-template-columns: 1fr;
		}

		.actions-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
</style>