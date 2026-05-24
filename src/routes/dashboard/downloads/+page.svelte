<script lang="ts">
	import { IconDownload, IconFileText, IconArchive } from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import EmptyState from '$lib/components/dashboard/EmptyState.svelte';

	const files = [
		{
			name: 'Options Greeks Guide',
			type: 'PDF',
			size: '1.2 MB',
			date: 'May 18, 2026',
			from: 'Free guide'
		},
		{
			name: 'Revolution Ranger · NT8 v2.4.1',
			type: 'ZIP',
			size: '8.4 MB',
			date: 'May 12, 2026',
			from: 'Indicator'
		},
		{
			name: 'Revolution Ranger · Pine v6 v2.4.1',
			type: 'TXT',
			size: '1.1 KB',
			date: 'May 12, 2026',
			from: 'Indicator'
		},
		{
			name: 'Revolution Ranger · TOS v2.4.1',
			type: 'TXT',
			size: '6.2 KB',
			date: 'May 12, 2026',
			from: 'Indicator'
		},
		{
			name: 'Q2 Macro Brief',
			type: 'PDF',
			size: '2.1 MB',
			date: 'Apr 28, 2026',
			from: 'Subscription'
		},
		{
			name: 'Revolution Ranger · Strategy Playbook',
			type: 'PDF',
			size: '4.8 MB',
			date: 'Nov 18, 2025',
			from: 'Indicator'
		}
	];
</script>

<Seo title="Downloads" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Library</p>
		<h2>Every file in your account.</h2>
		<p class="muted">{files.length} files · always download the latest version.</p>
	</div>
</header>

{#if files.length === 0}
	<EmptyState
		title="No downloads yet"
		description="Purchases and free guides appear here automatically."
	/>
{:else}
	<article class="card">
		<table>
			<thead>
				<tr>
					<th>File</th>
					<th>Type</th>
					<th>From</th>
					<th>Size</th>
					<th class="right">Date</th>
					<th></th>
				</tr>
			</thead>
			<tbody>
				{#each files as f}
					<tr>
						<td class="name-cell">
							<span class="ic">
								{#if f.type === 'PDF'}<IconFileText size={16} />{:else}<IconArchive
										size={16}
									/>{/if}
							</span>
							{f.name}
						</td>
						<td><Badge variant="outline" size="sm">{f.type}</Badge></td>
						<td class="muted">{f.from}</td>
						<td class="muted">{f.size}</td>
						<td class="right muted">{f.date}</td>
						<td class="right">
							<button class="dl-btn" type="button" aria-label="Download {f.name}">
								<IconDownload size={14} />
							</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</article>
{/if}

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

	.card {
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
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
		font-size: var(--text-sm);
		color: var(--ink-200);
		border-top: 1px solid var(--border-subtle);
		vertical-align: middle;
	}
	td.right {
		text-align: right;
	}
	td.muted {
		color: var(--ink-400);
	}
	.name-cell {
		color: var(--ink-100);
		font-weight: var(--weight-medium);
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.ic {
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		color: var(--gold-400);
		border-radius: var(--radius-full);
	}
	.dl-btn {
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
	.dl-btn:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
</style>
