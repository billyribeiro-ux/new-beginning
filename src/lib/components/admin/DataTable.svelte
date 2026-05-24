<script lang="ts" generics="T extends Record<string, unknown>">
	import type { Snippet } from 'svelte';
	import {
		IconArrowsSort,
		IconSortAscending,
		IconSortDescending,
		IconSearch
	} from '@tabler/icons-svelte';
	import EmptyState from '$lib/components/dashboard/EmptyState.svelte';

	type Column<Row> = {
		key: keyof Row & string;
		label: string;
		sortable?: boolean;
		align?: 'left' | 'right' | 'center';
		width?: string;
		render?: Snippet<[Row]>;
	};

	type Props = {
		rows: T[];
		columns: Column<T>[];
		rowKey: (row: T) => string;
		searchable?: boolean;
		searchKeys?: Array<keyof T & string>;
		pageSize?: number;
		emptyTitle?: string;
		emptyDescription?: string;
	};

	let {
		rows,
		columns,
		rowKey,
		searchable = true,
		searchKeys,
		pageSize = 10,
		emptyTitle = 'No records',
		emptyDescription = 'Adjust your filters or add new entries.'
	}: Props = $props();

	let query = $state('');
	let sortKey = $state<string | null>(null);
	let sortDir = $state<'asc' | 'desc'>('asc');
	let pageIndex = $state(0);

	function toggleSort(col: Column<T>) {
		if (!col.sortable) return;
		if (sortKey === col.key) {
			sortDir = sortDir === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = col.key;
			sortDir = 'asc';
		}
	}

	const filtered = $derived.by(() => {
		const q = query.trim().toLowerCase();
		if (!q) return rows;
		const keys = searchKeys ?? (columns.map((c) => c.key) as Array<keyof T & string>);
		return rows.filter((r) =>
			keys.some((k) =>
				String(r[k] ?? '')
					.toLowerCase()
					.includes(q)
			)
		);
	});

	const sorted = $derived.by(() => {
		if (!sortKey) return filtered;
		const dir = sortDir === 'asc' ? 1 : -1;
		return [...filtered].sort((a, b) => {
			const av = a[sortKey as keyof T];
			const bv = b[sortKey as keyof T];
			if (av === bv) return 0;
			if (av === null || av === undefined) return -1 * dir;
			if (bv === null || bv === undefined) return 1 * dir;
			return av < bv ? -1 * dir : 1 * dir;
		});
	});

	const pageCount = $derived(Math.max(1, Math.ceil(sorted.length / pageSize)));
	const safePage = $derived(Math.min(pageIndex, pageCount - 1));
	const visible = $derived(sorted.slice(safePage * pageSize, safePage * pageSize + pageSize));
</script>

<div class="dt-wrap">
	{#if searchable}
		<div class="toolbar">
			<div class="search">
				<IconSearch size={16} />
				<input
					type="search"
					placeholder="Search…"
					bind:value={query}
					aria-label="Search table"
					oninput={() => (pageIndex = 0)}
				/>
			</div>
			<p class="count">{sorted.length} {sorted.length === 1 ? 'record' : 'records'}</p>
		</div>
	{/if}

	{#if visible.length === 0}
		<EmptyState title={emptyTitle} description={emptyDescription} />
	{:else}
		<div class="dt-scroll">
			<table>
				<thead>
					<tr>
						{#each columns as col (col.key)}
							<th style:width={col.width} style:text-align={col.align ?? 'left'}>
								{#if col.sortable}
									<button
										class="sort"
										type="button"
										onclick={() => toggleSort(col)}
										aria-label="Sort by {col.label}"
									>
										{col.label}
										{#if sortKey === col.key}
											{#if sortDir === 'asc'}<IconSortAscending
													size={14}
												/>{:else}<IconSortDescending size={14} />{/if}
										{:else}
											<IconArrowsSort size={14} class="muted" />
										{/if}
									</button>
								{:else}
									{col.label}
								{/if}
							</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each visible as row (rowKey(row))}
						<tr>
							{#each columns as col (col.key)}
								<td style:text-align={col.align ?? 'left'}>
									{#if col.render}
										{@render col.render(row)}
									{:else}
										{String(row[col.key as keyof T] ?? '')}
									{/if}
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		{#if pageCount > 1}
			<nav class="pager" aria-label="Pagination">
				<button
					type="button"
					onclick={() => (pageIndex = Math.max(0, safePage - 1))}
					disabled={safePage === 0}>← Previous</button
				>
				<span>Page {safePage + 1} of {pageCount}</span>
				<button
					type="button"
					onclick={() => (pageIndex = Math.min(pageCount - 1, safePage + 1))}
					disabled={safePage === pageCount - 1}>Next →</button
				>
			</nav>
		{/if}
	{/if}
</div>

<style>
	.dt-wrap {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	.search {
		position: relative;
		flex: 1;
		max-width: 360px;
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		padding: 0 var(--space-3);
		color: var(--ink-400);
	}
	.search input {
		flex: 1;
		height: 40px;
		background: transparent;
		border: 0;
		color: var(--ink-100);
		font-family: inherit;
		font-size: var(--text-sm);
	}
	.search input:focus {
		outline: none;
	}
	.count {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
	}

	.dt-scroll {
		overflow-x: auto;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		background: var(--surface-1);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--text-sm);
	}
	thead {
		background: var(--surface-2);
		border-bottom: 1px solid var(--border-default);
		position: sticky;
		top: 0;
	}
	th {
		padding: var(--space-3) var(--space-4);
		font-weight: var(--weight-semibold);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		color: var(--ink-400);
		white-space: nowrap;
	}
	.sort {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: transparent;
		border: 0;
		font: inherit;
		color: inherit;
		text-transform: inherit;
		letter-spacing: inherit;
		cursor: pointer;
		padding: 4px;
		border-radius: var(--radius-xs);
	}
	.sort:hover {
		color: var(--ink-200);
		background: var(--surface-3);
	}
	tbody tr {
		border-top: 1px solid var(--border-subtle);
		transition: background var(--dur-fast) var(--ease-out);
	}
	tbody tr:hover {
		background: var(--surface-2);
	}
	td {
		padding: var(--space-3) var(--space-4);
		color: var(--ink-200);
		vertical-align: middle;
	}
	.pager {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
	.pager button {
		padding: var(--space-2) var(--space-3);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		color: var(--ink-200);
		font-size: var(--text-xs);
	}
	.pager button:hover:not(:disabled) {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
	.pager button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
