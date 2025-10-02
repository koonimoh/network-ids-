<script lang="ts">
	import { onMount } from 'svelte';

	interface BlockedIP {
		ip: string;
		reason: string;
		blocked_at: string;
		expires_at: string | null;
		notes: string | null;
	}

	let blocklist: BlockedIP[] = [];
	let loading = false;
	let showAddForm = false;
	
	// Form fields
	let newIP = '';
	let newReason = '';
	let newNotes = '';
	let expiresInHours: number | null = null;

	async function fetchBlocklist() {
		try {
			const res = await fetch('/api/blocklist');
			const data = await res.json();
			if (data.success) {
				blocklist = data.data || [];
			}
		} catch (err) {
			console.error('Failed to fetch blocklist:', err);
		}
	}

	async function addToBlocklist() {
		if (!newIP.trim() || !newReason.trim()) {
			alert('IP address and reason are required');
			return;
		}

		loading = true;
		try {
			const res = await fetch('/api/blocklist', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					ip: newIP.trim(),
					reason: newReason.trim(),
					notes: newNotes.trim() || null,
					expires_in_hours: expiresInHours
				})
			});

			const data = await res.json();
			if (data.success) {
				await fetchBlocklist();
				resetForm();
				showAddForm = false;
			} else {
				alert(data.error || 'Failed to add IP');
			}
		} catch (err) {
			console.error('Failed to add IP:', err);
			alert('Failed to add IP to blocklist');
		} finally {
			loading = false;
		}
	}

	async function removeFromBlocklist(ip: string) {
		if (!confirm(`Remove ${ip} from blocklist?`)) return;

		loading = true;
		try {
			const res = await fetch(`/api/blocklist/${ip}`, {
				method: 'DELETE'
			});

			const data = await res.json();
			if (data.success) {
				await fetchBlocklist();
			} else {
				alert(data.error || 'Failed to remove IP');
			}
		} catch (err) {
			console.error('Failed to remove IP:', err);
			alert('Failed to remove IP from blocklist');
		} finally {
			loading = false;
		}
	}

	function resetForm() {
		newIP = '';
		newReason = '';
		newNotes = '';
		expiresInHours = null;
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString('en-US', {
			dateStyle: 'short',
			timeStyle: 'short'
		});
	}

	function isExpired(expiresAt: string | null): boolean {
		if (!expiresAt) return false;
		return new Date(expiresAt) < new Date();
	}

	onMount(() => {
		fetchBlocklist();
		
		// Auto-refresh every 5 seconds to catch blocks from alert modal
		const interval = setInterval(fetchBlocklist, 5000);
		
		return () => clearInterval(interval);
	});
</script>

<div class="blocklist-manager card">
	<div class="header">
		<div>
			<h3>IP Blocklist</h3>
			<p class="subtitle">{blocklist.length} blocked IP{blocklist.length !== 1 ? 's' : ''}</p>
		</div>
		<button class="btn-primary" on:click={() => showAddForm = !showAddForm}>
			{showAddForm ? 'Cancel' : '+ Block IP'}
		</button>
	</div>

	{#if showAddForm}
		<form class="add-form" on:submit|preventDefault={addToBlocklist}>
			<div class="form-row">
				<div class="form-group">
					<label for="ip">IP Address *</label>
					<input 
						id="ip"
						type="text" 
						bind:value={newIP}
						placeholder="192.168.1.100"
						required
					/>
				</div>
				<div class="form-group">
					<label for="reason">Reason *</label>
					<input 
						id="reason"
						type="text" 
						bind:value={newReason}
						placeholder="Port scanning detected"
						required
					/>
				</div>
			</div>

			<div class="form-row">
				<div class="form-group">
					<label for="notes">Notes (Optional)</label>
					<input 
						id="notes"
						type="text" 
						bind:value={newNotes}
						placeholder="Additional details..."
					/>
				</div>
				<div class="form-group">
					<label for="expires">Expires In (hours)</label>
					<input 
						id="expires"
						type="number" 
						bind:value={expiresInHours}
						placeholder="Leave empty for permanent"
						min="1"
					/>
				</div>
			</div>

			<div class="form-actions">
				<button type="submit" class="btn-primary" disabled={loading}>
					{loading ? 'Adding...' : 'Add to Blocklist'}
				</button>
			</div>
		</form>
	{/if}

	{#if blocklist.length === 0}
		<div class="empty-state">
			<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="10"/>
				<line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/>
			</svg>
			<p>No blocked IPs</p>
		</div>
	{:else}
		<div class="blocklist-table">
			<table>
				<thead>
					<tr>
						<th>IP Address</th>
						<th>Reason</th>
						<th>Blocked At</th>
						<th>Expires</th>
						<th>Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each blocklist as entry}
						<tr class:expired={isExpired(entry.expires_at)}>
							<td class="mono">{entry.ip}</td>
							<td>
								<div class="reason-cell">
									<span class="reason">{entry.reason}</span>
									{#if entry.notes}
										<span class="notes" title={entry.notes}>
											<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
												<circle cx="12" cy="12" r="10"/>
												<line x1="12" y1="16" x2="12" y2="12"/>
												<line x1="12" y1="8" x2="12.01" y2="8"/>
											</svg>
										</span>
									{/if}
								</div>
							</td>
							<td class="date">{formatDate(entry.blocked_at)}</td>
							<td class="date">
								{#if entry.expires_at}
									<span class:expired-text={isExpired(entry.expires_at)}>
										{formatDate(entry.expires_at)}
									</span>
								{:else}
									<span class="permanent">Never</span>
								{/if}
							</td>
							<td>
								<button 
									class="btn-remove"
									on:click={() => removeFromBlocklist(entry.ip)}
									disabled={loading}
									title="Remove from blocklist"
								>
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<polyline points="3 6 5 6 21 6"/>
										<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
									</svg>
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.card {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1.5rem;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1.5rem;
	}

	h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		font-size: 0.875rem;
		color: var(--text-secondary);
	}

	.btn-primary {
		padding: 0.625rem 1.25rem;
		background: var(--accent-primary);
		color: var(--text-inverse);
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.add-form {
		padding: 1.5rem;
		background: var(--bg-tertiary);
		border-radius: 0.5rem;
		margin-bottom: 1.5rem;
	}

	.form-row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	input {
		padding: 0.625rem 0.875rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		color: var(--text-primary);
		font-size: 0.875rem;
	}

	input:focus {
		outline: none;
		border-color: var(--accent-primary);
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 1rem;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 3rem 2rem;
		color: var(--text-secondary);
	}

	.empty-state svg {
		margin-bottom: 1rem;
		opacity: 0.5;
	}

	.blocklist-table {
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
	}

	thead {
		background: var(--bg-tertiary);
	}

	th {
		padding: 0.75rem 1rem;
		text-align: left;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	tbody tr {
		border-bottom: 1px solid var(--border-primary);
	}

	tbody tr:hover {
		background: var(--bg-hover);
	}

	tbody tr.expired {
		opacity: 0.5;
	}

	td {
		padding: 1rem;
		font-size: 0.875rem;
	}

	.reason-cell {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.notes {
		color: var(--text-tertiary);
		cursor: help;
	}

	.date {
		color: var(--text-secondary);
		font-size: 0.8125rem;
	}

	.expired-text {
		color: var(--error);
	}

	.permanent {
		color: var(--text-tertiary);
		font-style: italic;
	}

	.btn-remove {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		border-radius: 0.375rem;
		color: var(--error);
		transition: all 0.2s;
	}

	.btn-remove:hover:not(:disabled) {
		background: color-mix(in srgb, var(--error) 20%, transparent);
	}

	.btn-remove:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	@media (max-width: 768px) {
		.blocklist-table {
			font-size: 0.8125rem;
		}

		th, td {
			padding: 0.5rem;
		}
	}
</style>