<script lang="ts">
	import {
		IconPlus,
		IconEdit,
		IconCopy,
		IconTrash,
		IconChartCandle,
		IconBook2,
		IconSearch
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { resolve } from '$app/paths';
	import { getAllProducts } from '$lib/data/products.js';
	import { formatPrice } from '$lib/utils/money.js';

	const rows = getAllProducts().map((p) => ({
		id: p.id,
		slug: p.slug,
		name: p.name,
		kind: p.kind,
		price: p.priceCents,
		updated: 'May 12, 2026'
	}));

	let query = $state('');
	const filtered = $derived(
		rows.filter(
			(r) =>
				!query.trim() ||
				r.name.toLowerCase().includes(query.toLowerCase()) ||
				r.slug.toLowerCase().includes(query.toLowerCase())
		)
	);
</script>

<Seo title="Admin · Products" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Catalog</p>
		<h2>Products</h2>
		<p class="muted">{rows.length} active products · indicators and courses.</p>
	</div>
	<Button variant="primary" size="md" href="/admin/products/new">
		{#snippet iconLeft()}<IconPlus size={14} />{/snippet}
		New product
	</Button>
</header>

<div class="toolbar">
	<div class="search">
		<IconSearch size={16} />
		<input
			type="search"
			placeholder="Search products…"
			bind:value={query}
			aria-label="Search products"
		/>
	</div>
	<p class="count">{filtered.length} of {rows.length}</p>
</div>

<article class="custom-table">
	<table>
		<thead>
			<tr>
				<th>Product</th>
				<th>Kind</th>
				<th class="right">Price</th>
				<th>Status</th>
				<th class="right">Updated</th>
				<th class="right">Actions</th>
			</tr>
		</thead>
		<tbody>
			{#each filtered as r (r.id)}
				<tr>
					<td>
						<div class="cell-product">
							<span class="ic" class:course={r.kind === 'course'}>
								{#if r.kind === 'indicator'}<IconChartCandle size={14} />{:else}<IconBook2
										size={14}
									/>{/if}
							</span>
							<div>
								<a href={resolve('/admin/products/[id]', { id: r.id })} class="pn">{r.name}</a>
								<p class="ps">{r.slug}</p>
							</div>
						</div>
					</td>
					<td><Badge variant="outline" size="sm">{r.kind}</Badge></td>
					<td class="right strong">{formatPrice(r.price)}</td>
					<td><Badge variant="success" size="sm">Active</Badge></td>
					<td class="right muted">{r.updated}</td>
					<td class="right">
						<div class="row-actions">
							<a class="ra" href={resolve('/admin/products/[id]', { id: r.id })} aria-label="Edit"
								><IconEdit size={14} /></a
							>
							<button class="ra" type="button" aria-label="Duplicate"><IconCopy size={14} /></button
							>
							<button class="ra ra-danger" type="button" aria-label="Delete"
								><IconTrash size={14} /></button
							>
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</article>

<style>
	.ph {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: var(--space-4);
		flex-wrap: wrap;
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	.eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.eyebrow::before {
		display: none;
	}
	.ph h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: var(--space-2) 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		margin-bottom: var(--space-4);
	}
	.search {
		flex: 1;
		max-width: 380px;
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

	.custom-table {
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		overflow: hidden;
	}
	table {
		width: 100%;
		border-collapse: collapse;
	}
	thead {
		background: var(--surface-2);
	}
	th {
		text-align: left;
		padding: var(--space-4);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	th.right {
		text-align: right;
	}
	td {
		padding: var(--space-4);
		border-top: 1px solid var(--border-subtle);
		font-size: var(--text-sm);
		color: var(--ink-200);
		vertical-align: middle;
	}
	td.right {
		text-align: right;
	}
	td.strong {
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
	}
	td.muted {
		color: var(--ink-400);
	}

	.cell-product {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.ic {
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: rgba(232, 182, 96, 0.12);
		color: var(--gold-300);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-full);
	}
	.ic.course {
		background: rgba(245, 208, 138, 0.08);
	}
	.pn {
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
		text-decoration: none;
	}
	.pn:hover {
		color: var(--gold-300);
	}
	.ps {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}

	.row-actions {
		display: inline-flex;
		gap: var(--space-1);
		justify-content: flex-end;
	}
	.ra {
		width: 30px;
		height: 30px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-300);
		cursor: pointer;
		text-decoration: none;
	}
	.ra:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
	.ra-danger:hover {
		color: var(--danger);
		border-color: var(--danger);
	}
</style>
