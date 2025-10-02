<script lang="ts">
	import { onMount } from 'svelte';
	import { aiConfig, chatHistory } from '$lib/stores/ai';
	import { wsStore } from '$lib/stores/websocket';
	import { AIService } from '$lib/services/ai';
	import type { SystemStats, ApiResponse, ThreatAlert } from '$lib/types';

	let stats: SystemStats | null = null;
	let showApiKeyInput = false;
	let tempApiKey = '';
	let userMessage = '';
	let isLoading = false;
	let error = '';
	let promptFile: FileList | null = null;
	let includeStats = true;
	let includeAlerts = true;

	$: isConfigured = $aiConfig.apiKey.length > 0;
	$: canSend = userMessage.trim().length > 0 && !isLoading && isConfigured;

	onMount(async () => {
		// Load current stats
		try {
			const res = await fetch('/api/stats');
			const data: ApiResponse<SystemStats> = await res.json();
			if (data.success && data.data) {
				stats = data.data;
			}
		} catch (err) {
			console.error('Failed to load stats:', err);
		}

		// Initialize with API key if already set
		if ($aiConfig.apiKey) {
			showApiKeyInput = false;
		} else {
			showApiKeyInput = true;
		}
	});

	function saveApiKey() {
		if (tempApiKey.trim()) {
			aiConfig.setApiKey(tempApiKey.trim());
			showApiKeyInput = false;
			tempApiKey = '';
		}
	}

	function clearApiKey() {
		if (confirm('Clear API key and chat history?')) {
			aiConfig.clear();
			chatHistory.clear();
			showApiKeyInput = true;
		}
	}

	async function handleFileUpload(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		try {
			const text = await file.text();
			userMessage = text;
		} catch (err) {
			error = 'Failed to read file';
		}
	}

	async function sendMessage() {
		if (!canSend) return;

		const message = userMessage.trim();
		userMessage = '';
		error = '';
		isLoading = true;

		// Add user message to history
		chatHistory.addMessage({
			role: 'user',
			content: message,
			timestamp: new Date()
		});

		try {
			// Build context from current data
			const contextStats = includeStats ? stats : null;
			const contextAlerts = includeAlerts ? $wsStore.alerts : [];
			const context = AIService.buildContext(contextStats, contextAlerts);

			// Create AI service instance
			const aiService = new AIService(
				$aiConfig.apiKey,
				$aiConfig.provider,
				$aiConfig.model
			);

			// Convert chat history to messages format
			const conversationHistory = $chatHistory.map(msg => ({
				role: msg.role,
				content: msg.content
			}));

			// Send message and get response
			const response = await aiService.sendMessage(message, context, conversationHistory);

			// Add assistant response to history
			chatHistory.addMessage({
				role: 'assistant',
				content: response,
				timestamp: new Date()
			});

		} catch (err: any) {
			error = err.message || 'Failed to get response from AI';
			console.error('AI request failed:', err);
		} finally {
			isLoading = false;
		}
	}

	function handleKeyPress(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			sendMessage();
		}
	}

	function clearChat() {
		if (confirm('Clear chat history?')) {
			chatHistory.clear();
		}
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
	}
</script>

<div class="ai-assistant-page">
	<div class="page-header">
		<div>
			<h1>AI Security Analyst</h1>
			<p class="subtitle">Ask questions about your IDS data using ChatGPT or Claude</p>
		</div>
		{#if isConfigured}
			<button class="btn-secondary" on:click={clearApiKey}>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M21 4H8l-7 8 7 8h13a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2z"/>
					<line x1="18" y1="9" x2="12" y2="15"/>
					<line x1="12" y1="9" x2="18" y2="15"/>
				</svg>
				Clear API Key
			</button>
		{/if}
	</div>

	{#if !isConfigured || showApiKeyInput}
		<div class="config-card card">
			<h3>Configure AI Provider</h3>
			
			<div class="provider-selector">
				<button 
					class="provider-btn"
					class:active={$aiConfig.provider === 'openai'}
					on:click={() => aiConfig.setProvider('openai')}
				>
					<div class="provider-icon">🤖</div>
					<div class="provider-info">
						<span class="provider-name">OpenAI</span>
						<span class="provider-model mono">GPT-4o</span>
					</div>
				</button>
				
				<button 
					class="provider-btn"
					class:active={$aiConfig.provider === 'anthropic'}
					on:click={() => aiConfig.setProvider('anthropic')}
				>
					<div class="provider-icon">🧠</div>
					<div class="provider-info">
						<span class="provider-name">Anthropic</span>
						<span class="provider-model mono">Claude 3.5 Sonnet</span>
					</div>
				</button>
			</div>

			<div class="form-group">
				<label for="api-key">
					{$aiConfig.provider === 'openai' ? 'OpenAI' : 'Anthropic'} API Key
				</label>
				<input 
					id="api-key"
					type="password" 
					bind:value={tempApiKey}
					placeholder="sk-..."
					on:keypress={(e) => e.key === 'Enter' && saveApiKey()}
				/>
				<span class="help-text">
					Get your API key from {$aiConfig.provider === 'openai' ? 'platform.openai.com' : 'console.anthropic.com'}
				</span>
			</div>

			<button class="btn-primary" on:click={saveApiKey} disabled={!tempApiKey.trim()}>
				Save API Key
			</button>
		</div>
	{:else}
		<div class="chat-layout">
			<div class="chat-sidebar card">
				<h3>Context Configuration</h3>
				
				<div class="context-options">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={includeStats} />
						<span>Include System Statistics</span>
					</label>
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={includeAlerts} />
						<span>Include Threat Alerts ({$wsStore.alerts.length})</span>
					</label>
				</div>

				<div class="stats-preview">
					<h4>Current Data Available</h4>
					{#if stats}
						<div class="preview-item">
							<span class="preview-label">Packets</span>
							<span class="preview-value mono">{stats.packets_processed.toLocaleString()}</span>
						</div>
						<div class="preview-item">
							<span class="preview-label">Threats</span>
							<span class="preview-value mono">{stats.threats_detected}</span>
						</div>
						<div class="preview-item">
							<span class="preview-label">Active Flows</span>
							<span class="preview-value mono">{stats.active_flows}</span>
						</div>
					{/if}
					<div class="preview-item">
						<span class="preview-label">Alerts</span>
						<span class="preview-value mono">{$wsStore.alerts.length}</span>
					</div>
				</div>

				<div class="model-info">
					<h4>Current Model</h4>
					<div class="model-badge">
						<span class="model-provider">{$aiConfig.provider === 'openai' ? 'OpenAI' : 'Anthropic'}</span>
						<span class="model-name mono">{$aiConfig.model}</span>
					</div>
				</div>

				{#if $chatHistory.length > 0}
					<button class="btn-secondary full-width" on:click={clearChat}>
						Clear Chat History
					</button>
				{/if}
			</div>

			<div class="chat-main card">
				<div class="chat-messages">
					{#if $chatHistory.length === 0}
						<div class="empty-chat">
							<svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
								<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
							</svg>
							<h3>Ask About Your Security Data</h3>
							<p>Examples:</p>
							<ul>
								<li>What are the most critical threats I should address?</li>
								<li>Analyze the network traffic patterns</li>
								<li>Are there any suspicious patterns in the alerts?</li>
								<li>What's the overall security posture?</li>
							</ul>
						</div>
					{:else}
						{#each $chatHistory as message}
							<div class="message" class:user={message.role === 'user'}>
								<div class="message-header">
									<span class="message-role">
										{message.role === 'user' ? '👤 You' : '🤖 AI Analyst'}
									</span>
									<span class="message-time mono">
										{message.timestamp.toLocaleTimeString()}
									</span>
								</div>
								<div class="message-content">
									{message.content}
								</div>
								<button 
									class="copy-btn"
									on:click={() => copyToClipboard(message.content)}
									title="Copy to clipboard"
								>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
										<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
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
										<span></span>
										<span></span>
										<span></span>
									</div>
								</div>
							</div>
						{/if}
					{/if}
				</div>

				{#if error}
					<div class="error-banner">
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<circle cx="12" cy="12" r="10"/>
							<line x1="12" y1="8" x2="12" y2="12"/>
							<line x1="12" y1="16" x2="12.01" y2="16"/>
						</svg>
						{error}
					</div>
				{/if}

				<div class="chat-input">
					<div class="input-toolbar">
						<label class="file-upload-btn">
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>
							</svg>
							<input 
								type="file" 
								accept=".txt,.md"
								on:change={handleFileUpload}
								hidden
							/>
						</label>
					</div>

					<textarea 
						bind:value={userMessage}
						placeholder="Ask a question about your IDS data..."
						on:keypress={handleKeyPress}
						disabled={isLoading}
						rows="3"
					></textarea>

					<button 
						class="send-btn"
						on:click={sendMessage}
						disabled={!canSend}
					>
						{#if isLoading}
							<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
								<path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
							</svg>
						{:else}
							<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<line x1="22" y1="2" x2="11" y2="13"/>
								<polygon points="22 2 15 22 11 13 2 9 22 2"/>
							</svg>
						{/if}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.ai-assistant-page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		height: calc(100vh - 8rem);
	}

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
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

	.config-card {
		max-width: 600px;
		margin: 0 auto;
	}

	.config-card h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 1.5rem;
	}

	.provider-selector {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.provider-btn {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		background: var(--bg-tertiary);
		border: 2px solid var(--border-primary);
		border-radius: 0.5rem;
		transition: all 0.2s;
	}

	.provider-btn:hover {
		border-color: var(--border-secondary);
		background: var(--bg-hover);
	}

	.provider-btn.active {
		border-color: var(--accent-primary);
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
	}

	.provider-icon {
		font-size: 2rem;
		flex-shrink: 0;
	}

	.provider-info {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		text-align: left;
	}

	.provider-name {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.provider-model {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1.5rem;
	}

	label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	input[type="password"],
	input[type="text"] {
		padding: 0.625rem 0.875rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
	}

	.help-text {
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	.btn-primary, .btn-secondary {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.625rem 1.25rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--accent-primary);
		color: var(--text-inverse);
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.btn-secondary {
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-primary);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--bg-hover);
		border-color: var(--border-secondary);
	}

	.btn-primary:disabled, .btn-secondary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.full-width {
		width: 100%;
	}

	.chat-layout {
		display: grid;
		grid-template-columns: 280px 1fr;
		gap: 1.5rem;
		flex: 1;
		min-height: 0;
	}

	.chat-sidebar {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		height: fit-content;
		position: sticky;
		top: 6rem;
	}

	.chat-sidebar h3 {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
	}

	.context-options {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		cursor: pointer;
		font-size: 0.875rem;
	}

	.checkbox-label input[type="checkbox"] {
		width: 1.125rem;
		height: 1.125rem;
		cursor: pointer;
	}

	.stats-preview h4,
	.model-info h4 {
		font-size: 0.8125rem;
		font-weight: 600;
		margin-bottom: 0.75rem;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.preview-item {
		display: flex;
		justify-content: space-between;
		padding: 0.5rem;
		background: var(--bg-tertiary);
		border-radius: 0.375rem;
		margin-bottom: 0.5rem;
	}

	.preview-label {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.preview-value {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.model-badge {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border-radius: 0.375rem;
	}

	.model-provider {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.model-name {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--accent-primary);
	}

	.chat-main {
		display: flex;
		flex-direction: column;
		min-height: 0;
		height: 100%;
	}

	.chat-messages {
		flex: 1;
		overflow-y: auto;
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		min-height: 400px;
	}

	.empty-chat {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		padding: 3rem 1rem;
		color: var(--text-secondary);
	}

	.empty-chat svg {
		margin-bottom: 1.5rem;
		opacity: 0.5;
	}

	.empty-chat h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 1rem;
		color: var(--text-primary);
	}

	.empty-chat p {
		margin-bottom: 0.5rem;
		font-weight: 500;
	}

	.empty-chat ul {
		list-style: none;
		text-align: left;
		max-width: 400px;
	}

	.empty-chat li {
		padding: 0.5rem 0;
		padding-left: 1.5rem;
		position: relative;
	}

	.empty-chat li::before {
		content: '→';
		position: absolute;
		left: 0;
		color: var(--accent-primary);
	}

	.message {
		padding: 1rem;
		background: var(--bg-tertiary);
		border-radius: 0.75rem;
		position: relative;
	}

	.message.user {
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
		margin-left: 2rem;
	}

	.message-header {
		display: flex;
		align-items: center;
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
		word-wrap: break-word;
	}

	.copy-btn {
		position: absolute;
		top: 0.75rem;
		right: 0.75rem;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.375rem;
		color: var(--text-secondary);
		opacity: 0;
		transition: all 0.2s;
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
		padding: 0.5rem 0;
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
		0%, 60%, 100% { opacity: 0.3; transform: translateY(0); }
		30% { opacity: 1; transform: translateY(-0.5rem); }
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.875rem 1rem;
		background: color-mix(in srgb, var(--error) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--error) 30%, transparent);
		border-radius: 0.5rem;
		color: var(--error);
		font-size: 0.875rem;
		margin: 0 1rem;
	}

	.chat-input {
		border-top: 1px solid var(--border-primary);
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.input-toolbar {
		display: flex;
		gap: 0.5rem;
	}

	.file-upload-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.25rem;
		height: 2.25rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.375rem;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.2s;
	}

	.file-upload-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-secondary);
	}

	textarea {
		width: 100%;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-family: inherit;
		color: var(--text-primary);
		resize: vertical;
		min-height: 80px;
		max-height: 200px;
	}

	textarea:focus {
		outline: none;
		border-color: var(--accent-primary);
	}

	textarea:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.send-btn {
		position: absolute;
		bottom: 1.75rem;
		right: 1.75rem;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
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
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	@media (max-width: 1024px) {
		.chat-layout {
			grid-template-columns: 1fr;
		}

		.chat-sidebar {
			position: static;
		}
	}
</style>