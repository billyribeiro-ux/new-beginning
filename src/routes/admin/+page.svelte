<script lang="ts">
	import type { PageData } from './$types';
	import { resolve } from '$app/paths';
	import IconCash from '@tabler/icons-svelte/icons/cash';
	import IconUsers from '@tabler/icons-svelte/icons/users';
	import IconCalendarStats from '@tabler/icons-svelte/icons/calendar-stats';
	import IconChartArea from '@tabler/icons-svelte/icons/chart-area';
	import IconArrowRight from '@tabler/icons-svelte/icons/arrow-right';
	import IconActivity from '@tabler/icons-svelte/icons/activity';
	import IconShoppingBag from '@tabler/icons-svelte/icons/shopping-bag';
	import IconUserPlus from '@tabler/icons-svelte/icons/user-plus';
	import IconMessages from '@tabler/icons-svelte/icons/messages';
	import Seo from '$lib/components/seo/Seo.svelte';
	import StatCard from '$lib/components/dashboard/StatCard.svelte';
	import ChartCard from '$lib/components/admin/ChartCard.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import EmptyState from '$lib/components/dashboard/EmptyState.svelte';
	import { stagger } from '$lib/animations/attachments.js';

	let { data }: { data: PageData } = $props();

	const revenue = [
		42_300, 48_900, 51_200, 53_800, 49_100, 58_600, 62_400, 65_900, 71_200, 68_400, 74_800, 82_100
	];
	const revLabels = [
		'Jun',
		'Jul',
		'Aug',
		'Sep',
		'Oct',
		'Nov',
		'Dec',
		'Jan',
		'Feb',
		'Mar',
		'Apr',
		'May'
	];

	const signups = [124, 138, 142, 156, 168, 172, 184, 192, 211, 224, 232, 248];

	const topProducts = [
		{ name: 'Day Trading · Quarterly', sales: 184, revenue: '$128,348' },
		{ name: 'Day Trading · Annual', sales: 92, revenue: '$183,724' },
		{ name: 'Revolution Ranger', sales: 76, revenue: '$75,772' },
		{ name: 'Options 101', sales: 64, revenue: '$63,808' },
		{ name: 'Day Trading · Monthly', sales: 412, revenue: '$101,764' }
	];

	const recentOrders = [
		{
			id: '#10248',
			cust: 'Alex Morgan',
			item: 'Day Trading · Quarterly',
			amount: '$697.00',
			when: '2 min ago'
		},
		{
			id: '#10247',
			cust: 'Priya Shah',
			item: 'Options 101',
			amount: '$997.00',
			when: '14 min ago'
		},
		{
			id: '#10246',
			cust: 'Daniel Reyes',
			item: 'Revolution Ranger',
			amount: '$997.00',
			when: '38 min ago'
		},
		{
			id: '#10245',
			cust: 'Selena Kim',
			item: 'Day Trading · Annual',
			amount: '$1,997.00',
			when: '1h ago'
		}
	];

	function fmtDate(ts: number | Date | null) {
		if (!ts) return '—';
		const d = ts instanceof Date ? ts : new Date(ts);
		return d.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<Seo title="Admin · Operations" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Operations</p>
		<h2>This is the desk view.</h2>
		<p class="muted">Last 30 days · auto-refreshes every 60s.</p>
	</div>
</header>

<section class="stats" {@attach stagger({ stagger: 0.08, y: 18 })}>
	<StatCard
		label="MRR"
		value={48720}
		prefix="$"
		deltaPct={12.4}
		hint="vs last month"
		icon={IconCash}
		spark={revenue.slice(-12).map((v) => v / 100)}
	/>
	<StatCard
		label="Active subscriptions"
		value={1284}
		deltaPct={4.2}
		hint="828 quarterly · 287 annual"
		icon={IconUsers}
		spark={[1100, 1135, 1142, 1158, 1172, 1196, 1218, 1240, 1258, 1267, 1273, 1284]}
	/>
	<StatCard
		label="Conversion rate"
		value={3.84}
		suffix="%"
		formatter={(n) => n.toFixed(2)}
		deltaPct={0.6}
		hint="of free-guide leads"
		icon={IconActivity}
	/>
	<StatCard
		label="New signups (mo)"
		value={248}
		deltaPct={6.9}
		hint="May 2026 to date"
		icon={IconUserPlus}
		spark={signups}
	/>
</section>

<div class="charts-row">
	<ChartCard
		title="Revenue trend"
		subtitle="Last 12 months · USD"
		data={revenue}
		labels={revLabels}
		height={260}
	/>
	<ChartCard
		title="New members"
		subtitle="Last 12 months"
		data={signups}
		labels={revLabels}
		height={260}
		color="#5DBB78"
	/>
</div>

<div class="grid-2">
	<section class="card">
		<header class="card-h">
			<h3><IconShoppingBag size={16} />Top products (this month)</h3>
			<a class="more" href={resolve('/admin/products')}>All products <IconArrowRight size={12} /></a
			>
		</header>
		<table>
			<thead>
				<tr><th>Product</th><th class="right">Sales</th><th class="right">Revenue</th></tr>
			</thead>
			<tbody>
				{#each topProducts as p (p.name)}
					<tr>
						<td>{p.name}</td>
						<td class="right">{p.sales}</td>
						<td class="right strong">{p.revenue}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</section>

	<section class="card">
		<header class="card-h">
			<h3><IconCalendarStats size={16} />Recent orders</h3>
			<a class="more" href={resolve('/admin/orders')}>All orders <IconArrowRight size={12} /></a>
		</header>
		<ul class="orders">
			{#each recentOrders as o (o.id)}
				<li>
					<div>
						<p class="ot">{o.cust}</p>
						<p class="oi">{o.id} · {o.item}</p>
					</div>
					<div class="oa">
						<strong>{o.amount}</strong>
						<span>{o.when}</span>
					</div>
				</li>
			{/each}
		</ul>
	</section>
</div>

<div class="grid-2">
	<section class="card">
		<header class="card-h">
			<h3><IconUserPlus size={16} />Recent leads (live from DB)</h3>
			<a class="more" href={resolve('/admin/leads')}>All leads <IconArrowRight size={12} /></a>
		</header>
		{#if data.recentLeads.length === 0}
			<EmptyState
				title="No leads yet"
				description="Email captures from the free-guide form will appear here."
			/>
		{:else}
			<ul class="leads">
				{#each data.recentLeads as l (l.id)}
					<li>
						<span class="lic">{l.email[0]?.toUpperCase()}</span>
						<div>
							<p class="le">{l.email}</p>
							<p class="ls">{l.source} · {fmtDate(l.createdAt)}</p>
						</div>
						<Badge variant="outline" size="sm">{l.source}</Badge>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="card">
		<header class="card-h">
			<h3><IconMessages size={16} />Recent messages</h3>
			<a class="more" href={resolve('/admin/messages')}>All messages <IconArrowRight size={12} /></a
			>
		</header>
		{#if data.recentMessages.length === 0}
			<EmptyState
				title="No messages yet"
				description="Contact form submissions will appear here."
			/>
		{:else}
			<ul class="leads">
				{#each data.recentMessages as m (m.id)}
					<li>
						<span class="lic">{m.name[0]?.toUpperCase()}</span>
						<div>
							<p class="le">{m.name} · {m.subject}</p>
							<p class="ls">{m.email} · {fmtDate(m.createdAt)}</p>
						</div>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.ph {
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

	.stats {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
		margin-bottom: clamp(1.5rem, 3vw, 2rem);
	}
	@media (--bp-md) {
		.stats {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (--bp-xl) {
		.stats {
			grid-template-columns: repeat(4, 1fr);
		}
	}

	.charts-row {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
		margin-bottom: clamp(1.5rem, 3vw, 2rem);
	}
	@media (--bp-xl) {
		.charts-row {
			grid-template-columns: 1.4fr 1fr;
		}
	}

	.grid-2 {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
		margin-bottom: var(--space-4);
	}
	@media (--bp-lg) {
		.grid-2 {
			grid-template-columns: 1fr 1fr;
		}
	}

	.card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.card-h {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-5);
	}
	.card-h h3 {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin: 0;
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.card-h h3 :global(svg) {
		color: var(--gold-400);
	}
	.more {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--text-xs);
		color: var(--ink-300);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		font-weight: var(--weight-semibold);
		text-decoration: none;
	}
	.more:hover {
		color: var(--gold-300);
	}

	table {
		width: 100%;
		border-collapse: collapse;
	}
	th {
		text-align: left;
		padding: 0 var(--space-3) var(--space-3);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		border-bottom: 1px solid var(--border-default);
	}
	th.right {
		text-align: right;
	}
	td {
		padding: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
		border-bottom: 1px solid var(--border-subtle);
	}
	td.right {
		text-align: right;
	}
	td.strong {
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
		font-family: var(--font-display);
	}

	.orders,
	.leads {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.orders li,
	.leads li {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
		align-items: center;
	}
	.leads li {
		grid-template-columns: 36px 1fr auto;
	}
	.lic {
		width: 36px;
		height: 36px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--gradient-gold);
		color: var(--surface-0);
		border-radius: var(--radius-full);
		font-weight: var(--weight-bold);
		font-size: var(--text-xs);
	}
	.ot,
	.le {
		font-size: var(--text-sm);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-medium);
	}
	.oi,
	.ls {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
	.oa {
		text-align: right;
	}
	.oa strong {
		font-family: var(--font-display);
		color: var(--ink-100);
		display: block;
	}
	.oa span {
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
</style>
