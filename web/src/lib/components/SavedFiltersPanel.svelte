<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { savedFiltersStore, type SavedFilter } from '$lib/stores/savedFilters';
	import type { AlertStatus } from '$lib/types';

	const dispatch = createEventDispatcher<{
		apply: SavedFilter['filters'];
	}>();

	let showCreateModal = false;
	let editingFilter: SavedFilter | null = null;
	
	// Form fields
	let filterName = '';
	let filterDescription = '';
	let filterColor = '#3b82f6';

	function applyFilter(filter: SavedFilter) {
		dispatch('apply', filter.filters);
	}

	function startCreate() {
		editingFilter = null;
		filterName = '';
		filterDescription = '';
		filterColor = '#3b82f6';
		showCreateModal = true;
	}

	function startEdit(filter: SavedFilter) {
		editingFilter = filter;
		filterName = filter.name;
		filterDescription = filter.description || '';
		filterColor = filter.color || '#3b82f6';
		showCreateModal = true;
	}

	function deleteFilter(id: string) {
		if (confirm('Delete this saved filter?')) {
			savedFiltersStore.deleteFilter(id);
		}
	}

	function isDefaultFilter(id: string): boolean {
		return ['critical-unresolved', 'high-investigating', 'port-scans', 'ddos-attacks'].includes(id);
	}

	function closeModal() {
		showCreateModal = false;
		editingFilter = null;
	}
</script>

<div class="saved-filters-panel">
	<div class="panel-header">
		<h4>Saved Filters</h4>
		<button class="btn-create" on:click={startCreate}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<line x1="12" y1="5" x2="12" y2="19"/>
				<line x1="5" y1="12" x2="19" y2="12"/>
			</svg>
			New
		</button>
	</div>

	<div class="filters-list">
		{#each $savedFiltersStore as filter (filter.id)}
			<button
				class="filter-item"
				on:click={() => applyFilter(filter)}
				style="--filter-color: {filter.color || '#3b82f6'}"
			>
				<div class="filter-indicator"></div>
				<div class="filter-content">
					<span class="filter-name">{filter.name}</span>
					{#if filter.description}
						<span class="filter-description">{filter.description}</span>
					{/if}
				</div>
				{#if !isDefaultFilter(filter.id)}
					<div class="filter-actions">
						<button
							class="action-btn"
							on:click|stopPropagation={() => startEdit(filter)}
							title="Edit"
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
								<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
							</svg>
						</button>
						<button
							class="action-btn delete"
							on:click|stopPropagation={() => deleteFilter(filter.id)}
							title="Delete"
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<polyline points="3 6 5 6 21 6"/>
								<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
							</svg>
						</button>
					</div>
				{/if}
			</button>
		{/each}
	</div>
</div>

{#if showCreateModal}
	<div class="modal-backdrop" on:click={closeModal}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h3>{editingFilter ? 'Edit Filter' : 'Create New Filter'}</h3>
				<button class="close-btn" on:click={closeModal}>
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<line x1="18" y1="6" x2="6" y2="18"/>
						<line x1="6" y1="6" x2="18" y2="18"/>
					</svg>
				</button>
			</div>

			<div class="modal-content">
				<p class="modal-description">
					{editingFilter 
						? 'Update the filter name, description, and color.' 
						: 'Apply your current filters, then save them with a name for quick access later.'}
				</p>

				<div class="form-group">
					<label for="filter-name">Filter Name *</label>
					<input
						id="filter-name"
						type="text"
						bind:value={filterName}
						placeholder="e.g., Critical Unresolved"
						required
					/>
				</div>

				<div class="form-group">
					<label for="filter-description">Description (optional)</label>
					<input
						id="filter-description"
						type="text"
						bind:value={filterDescription}
						placeholder="e.g., All critical threats that need attention"
					/>
				</div>

				<div class="form-group">
					<label for="filter-color">Color Tag</label>
					<div class="color-picker">
						<input
							id="filter-color"
							type="color"
							bind:value={filterColor}
						/>
						<span>{filterColor}</span>
					</div>
				</div>

				<div class="modal-actions">
					<button class="btn-secondary" on:click={closeModal}>
						Cancel
					</button>
					<button
						class="btn-primary"
						disabled={!filterName.trim()}
						on:click={() => {
							if (editingFilter) {
								savedFiltersStore.updateFilter(editingFilter.id, {
									name: filterName,
									description: filterDescription || undefined,
									color: filterColor
								});
							}
							closeModal();
						}}
					>
						{editingFilter ? 'Update' : 'Create'} Filter
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.saved-filters-panel {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		padding: 1rem;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
	}

	h4 {
		font-size: 0.9375rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.btn-create {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		padding: 0.375rem 0.75rem;
		background: var(--accent-primary);
		color: var(--text-inverse);
		border-radius: 0.375rem;
		font-size: 0.8125rem;
		font-weight: 600;
		transition: all 0.2s;
	}

	.btn-create:hover {
		background: var(--accent-hover);
	}

	.filters-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.filter-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		border-radius: 0.5rem;
		text-align: left;
		transition: all 0.2s;
		cursor: pointer;
	}

	.filter-item:hover {
		background: var(--bg-hover);
		border-color: var(--border-secondary);
	}

	.filter-indicator {
		width: 0.25rem;
		height: 2rem;
		background: var(--filter-color);
		border-radius: 0.125rem;
		flex-shrink: 0;
	}

	.filter-content {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		flex: 1;
		min-width: 0;
	}

	.filter-name {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.filter-description {
		font-size: 0.75rem;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.filter-actions {
		display: flex;
		gap: 0.25rem;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		border-radius: 0.25rem;
		background: transparent;
		color: var(--text-tertiary);
		transition: all 0.2s;
	}

	.action-btn:hover {
		background: var(--bg-secondary);
		color: var(--text-primary);
	}

	.action-btn.delete:hover {
		background: color-mix(in srgb, var(--error) 15%, transparent);
		color: var(--error);
	}

	/* Modal styles */
	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: 2rem;
		backdrop-filter: blur(4px);
	}

	.modal {
		background: var(--bg-secondary);
		border: 1px solid var(--border-primary);
		border-radius: 0.75rem;
		max-width: 500px;
		width: 100%;
		box-shadow: var(--shadow-lg);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.5rem;
		border-bottom: 1px solid var(--border-primary);
	}

	.modal-header h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		border-radius: 0.375rem;
		background: transparent;
		color: var(--text-secondary);
		transition: all 0.2s;
	}

	.close-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.modal-content {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
	}

	.modal-description {
		font-size: 0.875rem;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.form-group label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	.form-group input[type="text"] {
		padding: 0.625rem 0.875rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
	}

	.color-picker {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.color-picker input[type="color"] {
		width: 3rem;
		height: 2.5rem;
		border-radius: 0.5rem;
		cursor: pointer;
	}

	.color-picker span {
		font-size: 0.875rem;
		font-family: 'JetBrains Mono', monospace;
		color: var(--text-secondary);
	}

	.modal-actions {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
		margin-top: 0.5rem;
	}

	.btn-primary, .btn-secondary {
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

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: var(--bg-tertiary);
		border: 1px solid var(--border-primary);
		color: var(--text-primary);
	}

	.btn-secondary:hover {
		background: var(--bg-hover);
	}

	@media (max-width: 768px) {
		.modal-backdrop {
			padding: 0;
		}

		.modal {
			max-height: 100vh;
			border-radius: 0;
		}
	}
</style>