<script lang="ts">
	import IconSearch from '@tabler/icons-svelte/icons/search';
	import IconMail from '@tabler/icons-svelte/icons/mail';
	import IconEye from '@tabler/icons-svelte/icons/eye';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';

	const customers = [
		{
			id: 'usr_001',
			name: 'Alex Morgan',
			email: 'alex@example.com',
			joined: 'Nov 18, 2025',
			plan: 'Quarterly',
			ltv: 2391.0,
			status: 'active' as const
		},
		{
			id: 'usr_002',
			name: 'Priya Shah',
			email: 'priya@example.com',
			joined: 'Mar 4, 2026',
			plan: 'Annual',
			ltv: 2994.0,
			status: 'active' as const
		},
		{
			id: 'usr_003',
			name: 'Daniel Reyes',
			email: 'daniel@example.com',
			joined: 'Feb 12, 2026',
			plan: 'Monthly',
			ltv: 988.0,
			status: 'active' as const
		},
		{
			id: 'usr_004',
			name: 'Selena Kim',
			email: 'selena@example.com',
			joined: 'Jan 28, 2026',
			plan: 'Annual',
			ltv: 1997.0,
			status: 'active' as const
		},
		{
			id: 'usr_005',
			name: 'Marcus Anderson',
			email: 'marcus@example.com',
			joined: 'Dec 4, 2025',
			plan: 'Quarterly',
			ltv: 1394.0,
			status: 'paused' as const
		},
		{
			id: 'usr_006',
			name: 'Owen Tate',
			email: 'owen@example.com',
			joined: 'Apr 2, 2026',
			plan: '—',
			ltv: 997.0,
			status: 'one-off' as const
		},
		{
			id: 'usr_007',
			name: 'Casey Lin',
			email: 'casey@example.com',
			joined: 'Sep 14, 2025',
			plan: 'Annual',
			ltv: 4991.0,
			status: 'active' as const
		}
	];

	let query = $state('');
	const filtered = $derived(
		customers.filter(
			(c) =>
				!query.trim() ||
				c.name.toLowerCase().includes(query.toLowerCase()) ||
				c.email.toLowerCase().includes(query.toLowerCase())
		)
	);

	function statusVariant(s: string) {
		if (s === 'active') return 'success' as const;
		if (s === 'paused') return 'warning' as const;
		return 'default' as const;
	}

	function initials(name: string) {
		return name
			.split(' ')
			.map((p) => p[0])
			.slice(0, 2)
			.join('')
			.toUpperCase();
	}
</script>

<Seo title="Admin · Customers" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Sales</p>
		<h2>Customers</h2>
		<p class="muted">
			{customers.length} customers · {customers.filter((c) => c.status === 'active').length} active subscribers.
		</p>
	</div>
</header>

<div class="toolbar">
	<div class="search">
		<IconSearch size={16} />
		<input
			type="search"
			placeholder="Search customers…"
			bind:value={query}
			aria-label="Search customers"
		/>
	</div>
	<p class="count">{filtered.length} of {customers.length}</p>
</div>

<article class="table-card">
	<table>
		<thead>
			<tr>
				<th>Customer</th>
				<th>Plan</th>
				<th class="right">Lifetime value</th>
				<th>Joined</th>
				<th>Status</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each filtered as c (c.id)}
				<tr>
					<td>
						<div class="row">
							<span class="avatar">{initials(c.name)}</span>
							<div>
								<p class="cn">{c.name}</p>
								<p class="ce">{c.email}</p>
							</div>
						</div>
					</td>
					<td>{c.plan}</td>
					<td class="right strong">${c.ltv.toFixed(2)}</td>
					<td class="muted">{c.joined}</td>
					<td><Badge variant={statusVariant(c.status)} size="sm">{c.status}</Badge></td>
					<td class="right">
						<div class="row-acts">
							<button class="ra" type="button" aria-label="Send email"
								><IconMail size={14} /></button
							>
							<button class="ra" type="button" aria-label="View"><IconEye size={14} /></button>
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</article>

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

	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.avatar {
		width: 36px;
		height: 36px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		font-weight: var(--weight-bold);
		font-size: var(--text-xs);
	}
	.cn {
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.ce {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}

	.row-acts {
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
	}
	.ra:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
</style>
