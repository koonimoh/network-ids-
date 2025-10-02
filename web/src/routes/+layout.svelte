<script lang="ts">
	import { onMount } from 'svelte';
	import { theme } from '$lib/stores/theme';
	import { currentPage } from '$lib/stores/navigation';
	import type { Page } from '$lib/stores/navigation';
	import '../app.css';

	onMount(() => {
		theme.init();
	});

	function toggleTheme() {
		theme.toggle();
	}

	function navigateTo(page: Page) {
		currentPage.set(page);
	}
</script>

<div class="app">
	<header>
		<div class="container">
			<div class="header-content">
				<div class="logo">
					<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M12 2L2 7l10 5 10-5-10-5z"/>
						<path d="M2 17l10 5 10-5M2 12l10 5 10-5"/>
					</svg>
					<span class="mono">NETWORK_IDS</span>
				</div>

				<nav class="nav">
					<button 
						class="nav-item" 
						class:active={$currentPage === 'dashboard'}
						on:click={() => navigateTo('dashboard')}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<rect x="3" y="3" width="7" height="7"/>
							<rect x="14" y="3" width="7" height="7"/>
							<rect x="14" y="14" width="7" height="7"/>
							<rect x="3" y="14" width="7" height="7"/>
						</svg>
						<span>Dashboard</span>
					</button>
					
					<button 
						class="nav-item"
						class:active={$currentPage === 'analytics'}
						on:click={() => navigateTo('analytics')}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M21 21H4.6c-.56 0-.6-.44-.6-1V3"/>
							<path d="M9 18v-5M15 18v-9M21 18V9"/>
						</svg>
						<span>Analytics</span>
					</button>
					
					
					
					
					<button 
						class="nav-item"
						class:active={$currentPage === 'threat-map'}
						on:click={() => navigateTo('threat-map')}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<circle cx="12" cy="12" r="10"/>
							<line x1="2" y1="12" x2="22" y2="12"/>
							<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
						</svg>
						<span>Threat Map</span>
					</button>
					
					
					
					
					<button 
						class="nav-item"
						class:active={$currentPage === 'ai-assistant'}
						on:click={() => navigateTo('ai-assistant')}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
							<path d="M8 10h.01M12 10h.01M16 10h.01"/>
						</svg>
						<span>AI Assistant</span>
					</button>
					
					<button 
						class="nav-item"
						class:active={$currentPage === 'settings'}
						on:click={() => navigateTo('settings')}
					>
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
							<circle cx="12" cy="12" r="3"/>
						</svg>
						<span>Settings</span>
					</button>
				</nav>
				
				<button class="theme-toggle" on:click={toggleTheme} aria-label="Toggle theme">
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<circle cx="12" cy="12" r="5"/>
						<line x1="12" y1="1" x2="12" y2="3"/>
						<line x1="12" y1="21" x2="12" y2="23"/>
						<line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
						<line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
						<line x1="1" y1="12" x2="3" y2="12"/>
						<line x1="21" y1="12" x2="23" y2="12"/>
						<line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
						<line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
					</svg>
				</button>
			</div>
		</div>
	</header>

	<main>
		<div class="container">
			<slot />
		</div>
	</main>
</div>

<style>
	.app {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	header {
		position: sticky;
		top: 0;
		z-index: 100;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border-primary);
		backdrop-filter: saturate(180%) blur(20px);
	}

	.container {
		max-width: 1400px;
		margin: 0 auto;
		padding: 0 1.5rem;
		width: 100%;
	}

	.header-content {
		display: flex;
		align-items: center;
		gap: 2rem;
		height: 4rem;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		font-size: 1rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.logo svg {
		color: var(--accent-primary);
	}

	.nav {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex: 1;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		border-radius: 0.5rem;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s;
	}

	.nav-item:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.nav-item.active {
		background: var(--accent-primary);
		color: var(--text-inverse);
	}

	.nav-item svg {
		width: 18px;
		height: 18px;
	}

	.theme-toggle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		border-radius: 0.5rem;
		background: transparent;
		color: var(--text-secondary);
		margin-left: auto;
	}

	.theme-toggle:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	main {
		flex: 1;
		padding: 2rem 0;
	}

	@media (max-width: 768px) {
		.container {
			padding: 0 1rem;
		}
		
		.header-content {
			gap: 1rem;
		}

		.nav {
			gap: 0.25rem;
		}

		.nav-item span {
			display: none;
		}

		.nav-item {
			padding: 0.5rem;
		}
		
		main {
			padding: 1.5rem 0;
		}
	}
</style>