<script lang="ts">
	import IconSearch from '@tabler/icons-svelte/icons/search';
	import IconDownload from '@tabler/icons-svelte/icons/download';
	import IconEye from '@tabler/icons-svelte/icons/eye';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';

	const orders = [
		{
			id: '#10248',
			cust: 'Alex Morgan',
			email: 'alex@example.com',
			item: 'Day Trading · Quarterly',
			amount: 697.0,
			status: 'paid',
			date: 'May 18, 2026 · 14:24'
		},
		{
			id: '#10247',
			cust: 'Priya Shah',
			email: 'priya@example.com',
			item: 'Options 101',
			amount: 997.0,
			status: 'paid',
			date: 'May 18, 2026 · 14:10'
		},
		{
			id: '#10246',
			cust: 'Daniel Reyes',
			email: 'daniel@example.com',
			item: 'Revolution Ranger',
			amount: 997.0,
			status: 'paid',
			date: 'May 18, 2026 · 13:46'
		},
		{
			id: '#10245',
			cust: 'Selena Kim',
			email: 'selena@example.com',
			item: 'Day Trading · Annual',
			amount: 1997.0,
			status: 'paid',
			date: 'May 18, 2026 · 13:18'
		},
		{
			id: '#10244',
			cust: 'Marcus Anderson',
			email: 'marcus@example.com',
			item: 'Day Trading · Monthly',
			amount: 247.0,
			status: 'pending',
			date: 'May 18, 2026 · 12:54'
		},
		{
			id: '#10243',
			cust: 'Owen Tate',
			email: 'owen@example.com',
			item: 'Options 101',
			amount: 997.0,
			status: 'refunded',
			date: 'May 18, 2026 · 11:32'
		},
		{
			id: '#10242',
			cust: 'Casey Lin',
			email: 'casey@example.com',
			item: 'Revolution Ranger',
			amount: 997.0,
			status: 'paid',
			date: 'May 18, 2026 · 11:08'
		}
	];

	let query = $state('');
	let statusFilter = $state<'all' | 'paid' | 'pending' | 'refunded'>('all');

	const filtered = $derived(
		orders.filter((o) => {
			if (statusFilter !== 'all' && o.status !== statusFilter) return false;
			if (query.trim()) {
				const q = query.toLowerCase();
				return (
					o.id.includes(q) || o.cust.toLowerCase().includes(q) || o.email.toLowerCase().includes(q)
				);
			}
			return true;
		})
	);

	function statusVariant(s: string) {
		if (s === 'paid') return 'success' as const;
		if (s === 'pending') return 'warning' as const;
		return 'danger' as const;
	}
</script>

<Seo title="Admin · Orders" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Sales</p>
		<h2>Orders</h2>
		<p class="muted">{orders.length} orders this week · 87% paid · 2 pending review.</p>
	</div>
	<button class="export">
		<IconDownload size={14} />
		Export CSV
	</button>
</header>

<div class="toolbar">
	<div class="search">
		<IconSearch size={16} />
		<input
			type="search"
			placeholder="Search by order, customer, or email…"
			bind:value={query}
			aria-label="Search orders"
		/>
	</div>
	<div class="filters">
		{#each ['all', 'paid', 'pending', 'refunded'] as s (s)}
			<button
				class="chip"
				class:is-active={statusFilter === s}
				type="button"
				onclick={() => (statusFilter = s as typeof statusFilter)}
			>
				{s.charAt(0).toUpperCase() + s.slice(1)}
			</button>
		{/each}
	</div>
</div>

<article class="table-card">
	<table>
		<thead>
			<tr>
				<th>Order</th>
				<th>Customer</th>
				<th>Item</th>
				<th class="right">Amount</th>
				<th>Status</th>
				<th class="right">Date</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each filtered as o (o.id)}
				<tr>
					<td class="mono">{o.id}</td>
					<td>
						<p class="cn">{o.cust}</p>
						<p class="ce">{o.email}</p>
					</td>
					<td>{o.item}</td>
					<td class="right strong">${o.amount.toFixed(2)}</td>
					<td><Badge variant={statusVariant(o.status)} size="sm">{o.status}</Badge></td>
					<td class="right muted">{o.date}</td>
					<td class="right">
						<button class="view-btn" type="button" aria-label="View order">
							<IconEye size={14} />
						</button>
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
	.export {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: var(--space-3) var(--space-4);
		background: var(--surface-2);
		color: var(--ink-200);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		cursor: pointer;
	}
	.export:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		margin-bottom: var(--space-4);
		flex-wrap: wrap;
	}
	.search {
		flex: 1;
		min-width: 240px;
		max-width: 480px;
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
	.filters {
		display: flex;
		gap: 4px;
		padding: 4px;
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
	}
	.chip {
		padding: 6px 14px;
		font-size: var(--text-xs);
		font-weight: var(--weight-medium);
		color: var(--ink-300);
		background: transparent;
		border: 0;
		border-radius: var(--radius-full);
		cursor: pointer;
		text-transform: capitalize;
	}
	.chip:hover {
		color: var(--ink-100);
	}
	.chip.is-active {
		background: var(--gradient-gold);
		color: var(--surface-0);
		font-weight: var(--weight-semibold);
		box-shadow: 0 4px 10px rgba(212, 162, 76, 0.32);
	}

	.table-card {
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
	td.muted {
		color: var(--ink-400);
	}
	td.strong {
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
	}
	td.mono {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--gold-400);
	}
	.cn {
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
	}
	.ce {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
	.view-btn {
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-300);
	}
	.view-btn:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
</style>
