<script lang="ts">
	import type { PageData } from './$types';
	import { IconSearch, IconDownload, IconMail, IconUserPlus } from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import EmptyState from '$lib/components/dashboard/EmptyState.svelte';

	let { data }: { data: PageData } = $props();
	let query = $state('');

	const filtered = $derived(
		data.leads.filter((l) => !query.trim() || l.email.toLowerCase().includes(query.toLowerCase()))
	);

	function fmtDate(ts: Date | number) {
		const d = ts instanceof Date ? ts : new Date(ts);
		return d.toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<Seo title="Admin · Leads" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Audience</p>
		<h2>Lead inbox</h2>
		<p class="muted">{data.leads.length} captures · synced live from Drizzle.</p>
	</div>
	<button class="export" type="button">
		<IconDownload size={14} />
		Export CSV
	</button>
</header>

<div class="toolbar">
	<div class="search">
		<IconSearch size={16} />
		<input
			type="search"
			placeholder="Search by email…"
			bind:value={query}
			aria-label="Search leads"
		/>
	</div>
	<p class="count">{filtered.length} of {data.leads.length}</p>
</div>

{#if data.leads.length === 0}
	<EmptyState
		icon={IconUserPlus}
		title="No leads yet"
		description="Submit the Free Greeks Guide form on the public site to see the first capture appear here."
	/>
{:else}
	<article class="table-card">
		<table>
			<thead>
				<tr><th>Email</th><th>Source</th><th class="right">Captured</th><th></th></tr>
			</thead>
			<tbody>
				{#each filtered as l (l.id)}
					<tr>
						<td>
							<div class="row">
								<span class="avatar">{l.email[0]?.toUpperCase()}</span>
								<span class="email">{l.email}</span>
							</div>
						</td>
						<td><Badge variant="outline" size="sm">{l.source}</Badge></td>
						<td class="right muted">{fmtDate(l.createdAt)}</td>
						<td class="right">
							<a class="ra" href="mailto:{l.email}" aria-label="Email {l.email}">
								<IconMail size={14} />
							</a>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</article>
{/if}

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
		font-size: var(--text-xs);
	}
	.row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.avatar {
		width: 32px;
		height: 32px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		font-weight: var(--weight-bold);
		font-size: var(--text-xs);
	}
	.email {
		color: var(--ink-100);
		font-weight: var(--weight-medium);
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
		text-decoration: none;
	}
	.ra:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
</style>
