<script lang="ts">
	import { wsStore } from '$lib/stores/websocket';
	import { alertStatusStore } from '$lib/stores/alertStatus';  
	import type { AlertStatus } from '$lib/types'
	import type { ThreatAlert } from '$lib/types';
	import AlertDetailModal from './AlertDetailModal.svelte';
	import SavedFiltersPanel from './SavedFiltersPanel.svelte';

	let selectedAlert: ThreatAlert | null = null;
	let modalOpen = false;
	let modalAlert: ThreatAlert | null = null;
	let searchQuery = '';
	let severityFilter: 'All' | 'Critical' | 'High' | 'Medium' | 'Low' = 'All';
	let sortOrder: 'newest' | 'oldest' | 'severity' = 'newest';
	let statusFilter: 'all' | AlertStatus = 'all';

	function getSeverityColor(severity: string): string {
		switch (severity) {
			case 'Critical': return 'var(--severity-critical)';
			case 'High': return 'var(--severity-high)';
			case 'Medium': return 'var(--severity-medium)';
			case 'Low': return 'var(--severity-low)';
			default: return 'var(--text-secondary)';
		}
	}

	function formatTimestamp(timestamp: string): string {
		const date = new Date(timestamp);
		return date.toLocaleTimeString('en-US', { 
			hour: '2-digit', 
			minute: '2-digit',
			second: '2-digit'
		});
	}

	function selectAlert(alert: ThreatAlert) {
		selectedAlert = selectedAlert?.id === alert.id ? null : alert;
	}
	
	function getStatusLabel(status: AlertStatus): string {  
		switch (status) {
			case 'new': return 'New';
			case 'reviewed': return 'Reviewed';
			case 'investigating': return 'Investigating';
			case 'resolved': return 'Resolved';
			case 'false_positive': return 'False Positive';
		}
	}

	function getStatusColor(status: AlertStatus): string {  
		switch (status) {
			case 'new': return 'var(--text-secondary)';
			case 'reviewed': return 'var(--info)';
			case 'investigating': return 'var(--warning)';
			case 'resolved': return 'var(--success)';
			case 'false_positive': return 'var(--text-tertiary)';
		}
	}

	function openModal(alert: ThreatAlert) {
		modalAlert = alert;
		modalOpen = true;
	}

	function closeModal() {
		modalOpen = false;
		modalAlert = null;
	}

	function clearFilters() {
		searchQuery = '';
		severityFilter = 'All';
		sortOrder = 'newest';
		statusFilter = 'all';
	}
	
	
	
	function applyFilter(filters: any) { 
		searchQuery = filters.searchQuery;
		severityFilter = filters.severityFilter;
		statusFilter = filters.statusFilter;
		sortOrder = filters.sortOrder;
	}

	$: filteredAlerts = $wsStore.alerts.filter(alert => {
		// Search filter
		const matchesSearch = searchQuery === '' || 
			alert.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
			alert.source_ip.toLowerCase().includes(searchQuery.toLowerCase()) ||
			alert.threat_type.toLowerCase().includes(searchQuery.toLowerCase());

		// Severity filter
		const matchesSeverity = severityFilter === 'All' || alert.severity === severityFilter;
		
		// Status filter
		const alertStatus = alertStatusStore.getStatus(alert);
		const matchesStatus = statusFilter === 'all' || alertStatus === statusFilter;

		return matchesSearch && matchesSeverity && matchesStatus;;
	}).sort((a, b) => {
		switch (sortOrder) {
			case 'newest':
				return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
			case 'oldest':
				return new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime();
			case 'severity': {
				const severityOrder = { Critical: 4, High: 3, Medium: 2, Low: 1 };
				return severityOrder[b.severity] - severityOrder[a.severity];
			}
			default:
				return 0;
		}
	});
</script>

<div class="alert-feed card">
	<div class="feed-header">
		<div class="header-left">
			<h3>Threat Alerts</h3>
			<div class="feed-status">
				<div class="status-indicator" class:connected={$wsStore.connected}></div>
				<span>{$wsStore.connected ? 'LIVE' : 'DISCONNECTED'}</span>
			</div>
		</div>
		<div class="header-right">
			<span class="alert-count">{filteredAlerts.length} of {$wsStore.alerts.length}</span>
		</div>
	</div>
	
	<SavedFiltersPanel on:apply={(e) => applyFilter(e.detail)} />

	<div class="feed-controls">
		<div class="search-box">
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="11" cy="11" r="8"/>
				<path d="m21 21-4.35-4.35"/>
			</svg>
			<input 
				type="text" 
				placeholder="Search alerts..." 
				bind:value={searchQuery}
			/>
			{#if searchQuery}
				<button class="clear-btn" on:click={() => searchQuery = ''}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<line x1="18" y1="6" x2="6" y2="18"/>
						<line x1="6" y1="6" x2="18" y2="18"/>
					</svg>
				</button>
			{/if}
		</div>

		<div class="filter-group">
			<select bind:value={severityFilter} class="filter-select">
				<option value="All">All Severities</option>
				<option value="Critical">Critical</option>
				<option value="High">High</option>
				<option value="Medium">Medium</option>
				<option value="Low">Low</option>
			</select>

			<select bind:value={sortOrder} class="filter-select">
				<option value="newest">Newest First</option>
				<option value="oldest">Oldest First</option>
				<option value="severity">By Severity</option>
			</select>
			
			<select bind:value={statusFilter} class="filter-select">
				<option value="all">All Statuses</option>
				<option value="new">New</option>
				<option value="reviewed">Reviewed</option>
				<option value="investigating">Investigating</option>
				<option value="resolved">Resolved</option>
				<option value="false_positive">False Positive</option>
			</select>

			{#if searchQuery || severityFilter !== 'All' || sortOrder !== 'newest' || statusFilter !== 'all'}
				<button class="clear-filters-btn" on:click={clearFilters}>
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
					</svg>
				</button>
			{/if}
		</div>
	</div>

	<div class="alerts-container">
		{#if filteredAlerts.length === 0}
			{#if $wsStore.alerts.length === 0}
				<div class="empty-state">
					<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
					</svg>
					<p>No threats detected</p>
					<span>System is monitoring network traffic</span>
				</div>
			{:else}
				<div class="empty-state">
					<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<circle cx="11" cy="11" r="8"/>
						<path d="m21 21-4.35-4.35"/>
					</svg>
					<p>No alerts match your filters</p>
					<button class="link-btn" on:click={clearFilters}>Clear filters</button>
				</div>
			{/if}
		{:else}
			<div class="alert-list">
				{#each filteredAlerts as alert (alert.id)}
					{@const alertStatus = alertStatusStore.getStatus(alert)}
					<div 
						class="alert-item" 
						class:selected={selectedAlert?.id === alert.id}
						on:click={() => selectAlert(alert)}
						on:keydown={(e) => e.key === 'Enter' && selectAlert(alert)}
						role="button"
						tabindex="0"
					>
						<div class="alert-header">
							<div class="alert-severity" style="--severity-color: {getSeverityColor(alert.severity)}"> 
								<div class="severity-dot"></div>
								<span>{alert.severity.toUpperCase()}</span>
							</div>
							<div class="status-badge" style="--status-color: {getStatusColor(alertStatus)}">
								{getStatusLabel(alertStatus)}
							</div>
							<span class="alert-time mono">{formatTimestamp(alert.timestamp)}</span>
						</div>
						
						<div class="alert-content">
							<h4>{alert.threat_type}</h4>
							<p>{alert.description}</p>
						</div>

						<div class="alert-meta">
							<div class="meta-tag">
								<span class="mono">{alert.source_ip}</span>
							</div>
							{#if alert.target_ip}
								<span class="meta-arrow">→</span>
								<div class="meta-tag">
									<span class="mono">{alert.target_ip}</span>
								</div>
							{/if}
							<div class="meta-confidence">
								<span>{(alert.confidence * 100).toFixed(0)}%</span>
							</div>
							<button 
								class="view-details-btn"
								on:click|stopPropagation={() => openModal(alert)}
								title="View details"
							>
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
									<circle cx="12" cy="12" r="3"/>
								</svg>
							</button>
						</div>

						{#if selectedAlert?.id === alert.id}
							<div class="alert-details">
								<div class="detail-section">
									<h5>Primary Indicators</h5>
									<ul>
										{#each alert.explanation.primary_indicators as indicator}
											<li>{indicator}</li>
										{/each}
									</ul>
								</div>

								{#if alert.explanation.recommended_actions.length > 0}
									<div class="detail-section">
										<h5>Recommended Actions</h5>
										<ul>
											{#each alert.explanation.recommended_actions as action}
												<li>{action}</li>
											{/each}
										</ul>
									</div>
								{/if}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

{#if modalOpen && modalAlert}
	<AlertDetailModal alert={modalAlert} open={modalOpen} on:close={closeModal} />
{/if}

<style>
	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.feed-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.feed-header h3 {
		font-size: 1.125rem;
		font-weight: 600;
	}

	.feed-status {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

	.status-indicator {
		width: 0.5rem;
		height: 0.5rem;
		border-radius: 50%;
		background: var(--text-tertiary);
	}

	.status-indicator.connected {
		background: var(--success);
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}

	.alert-count {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-secondary);
		padding: 0.25rem 0.625rem;
		background: var(--bg-tertiary);
		border-radius: 0.25rem;
	}

	.feed-controls {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}

	.search-box {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-box svg:first-child {
		position: absolute;
		left: 0.875rem;
		color: var(--text-tertiary);
		pointer-events: none;
	}

	.search-box input {
		width: 100%;
		padding: 0.625rem 0.875rem 0.625rem 2.5rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
	}

	.clear-btn {
		position: absolute;
		right: 0.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		border-radius: 0.25rem;
		background: transparent;
		color: var(--text-tertiary);
		transition: all 0.2s;
	}

	.clear-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.filter-group {
		display: flex;
		gap: 0.5rem;
	}

	.filter-select {
		flex: 1;
		padding: 0.625rem 0.875rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
	}

	.clear-filters-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		padding: 0.625rem;
		border-radius: 0.5rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.clear-filters-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-secondary);
	}

	.alerts-container {
		max-height: 600px;
		overflow-y: auto;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 3rem 1rem;
		text-align: center;
		color: var(--text-secondary);
	}

	.empty-state svg {
		margin-bottom: 1rem;
		opacity: 0.5;
	}

	.empty-state p {
		font-size: 1rem;
		font-weight: 500;
		margin-bottom: 0.25rem;
		color: var(--text-primary);
	}

	.empty-state span {
		font-size: 0.875rem;
		margin-bottom: 1rem;
	}

	.link-btn {
		background: transparent;
		color: var(--accent-primary);
		font-size: 0.875rem;
		font-weight: 600;
		padding: 0.5rem 1rem;
		border-radius: 0.375rem;
		transition: all 0.2s;
	}

	.link-btn:hover {
		background: color-mix(in srgb, var(--accent-primary) 10%, transparent);
	}

	.alert-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.alert-item {
		padding: 1rem;
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.alert-item:hover {
		border-color: var(--border-secondary);
		background: var(--bg-tertiary);
	}

	.alert-item.selected {
		border-color: var(--accent-primary);
	}

	.alert-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 0.75rem;
	}

	.alert-severity {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		color: var(--severity-color);
	}

	.severity-dot {
		width: 0.5rem;
		height: 0.5rem;
		border-radius: 50%;
		background: var(--severity-color);
	}

	.alert-time {
		font-size: 0.75rem;
		color: var(--text-tertiary);
	}

	.alert-content h4 {
		font-size: 0.9375rem;
		font-weight: 600;
		margin-bottom: 0.25rem;
	}

	.alert-content p {
		font-size: 0.875rem;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.alert-meta {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.75rem;
		flex-wrap: wrap;
	}

	.meta-tag {
		padding: 0.25rem 0.625rem;
		background: var(--bg-tertiary);
		border-radius: 0.25rem;
		font-size: 0.75rem;
	}

	.meta-arrow {
		color: var(--text-tertiary);
		font-size: 0.875rem;
	}

	.meta-confidence {
		margin-left: auto;
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--accent-primary);
	}

	.view-details-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.25rem;
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.view-details-btn:hover {
		background: var(--accent-primary);
		border-color: var(--accent-primary);
		color: var(--text-inverse);
	}

	.alert-details {
		margin-top: 1rem;
		padding-top: 1rem;
		border-top: 1px solid var(--border-primary);
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.detail-section h5 {
		font-size: 0.8125rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.detail-section ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.detail-section li {
		font-size: 0.875rem;
		color: var(--text-secondary);
		padding-left: 1rem;
		position: relative;
	}

	.detail-section li::before {
		content: '•';
		position: absolute;
		left: 0;
		color: var(--accent-primary);
	}
	
	
	.status-badge {
		padding: 0.25rem 0.625rem;
		background: color-mix(in srgb, var(--status-color) 15%, transparent);
		border: 1px solid var(--status-color);
		border-radius: 0.25rem;
		font-size: 0.6875rem;
		font-weight: 600;
		color: var(--status-color);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	@media (max-width: 768px) {
		.filter-group {
			flex-direction: column;
		}
	}
	
	
	
</style>