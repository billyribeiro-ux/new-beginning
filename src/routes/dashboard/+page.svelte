<script lang="ts">
	import type { PageData } from './$types';
	import {
		IconChartLine,
		IconBook2,
		IconChartCandle,
		IconWallet,
		IconArrowRight,
		IconCalendarStats,
		IconDownload,
		IconPlayerPlay
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import StatCard from '$lib/components/dashboard/StatCard.svelte';
	import ProgressBar from '$lib/components/dashboard/ProgressBar.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { stagger } from '$lib/animations/attachments.js';
	import { resolve } from '$app/paths';

	let { data }: { data: PageData } = $props();

	const continueLearning = [
		{
			title: 'Options 101 · Module 3: Delta',
			progress: 65,
			last: '2 days ago',
			href: '/dashboard/courses/options-101'
		},
		{
			title: 'Revolution Ranger · Onboarding',
			progress: 100,
			last: 'Complete',
			href: '/dashboard/indicators'
		},
		{
			title: 'Day Trading Desk · Weekly review',
			progress: 30,
			last: 'In progress',
			href: '/dashboard/subscription'
		}
	];

	const upcomingCharges = [
		{ label: 'Day Trading · Quarterly', date: 'Jun 24, 2026', amount: '$697.00' },
		{ label: 'Revolution Ranger · annual support', date: 'Aug 12, 2026', amount: '$0.00' }
	];

	const recentDownloads = [
		{ name: 'Options Greeks Guide', date: 'May 18, 2026', kind: 'PDF', size: '1.2 MB' },
		{ name: 'Revolution Ranger · NT8 v2.4', date: 'May 12, 2026', kind: 'ZIP', size: '8.4 MB' },
		{ name: 'Q2 Macro Brief', date: 'Apr 28, 2026', kind: 'PDF', size: '2.1 MB' }
	];
</script>

<Seo title="Overview" noindex />

<div class="hero">
	<div>
		<p class="eyebrow">Welcome back</p>
		<h2>Hello, {data.user?.name.split(' ')[0] ?? 'trader'} — the desk opens in 14 minutes.</h2>
		<p class="muted">Here is where you stand this week.</p>
	</div>
	<div class="hero-actions">
		<Button variant="gold-outline" size="md" href="/dashboard/subscription"
			>View subscription</Button
		>
		<Button variant="primary" size="md" href="/dashboard/courses">
			Continue learning
			{#snippet iconRight()}<IconArrowRight size={14} />{/snippet}
		</Button>
	</div>
</div>

<section class="stats" {@attach stagger({ stagger: 0.08, y: 18 })}>
	<StatCard
		label="Active subscription"
		value={697}
		prefix="$"
		suffix=""
		hint="Quarterly · renews Jun 24"
		icon={IconWallet}
		deltaPct={0}
	/>
	<StatCard label="Courses in progress" value={1} hint="Options 101" icon={IconBook2} />
	<StatCard label="Indicators active" value={1} hint="Revolution Ranger" icon={IconChartCandle} />
	<StatCard
		label="Days on the desk"
		value={184}
		suffix=""
		hint="Member since Nov 2025"
		icon={IconCalendarStats}
		spark={[12, 18, 14, 22, 28, 24, 30, 35, 32, 40, 38, 44]}
		deltaPct={4.2}
	/>
</section>

<div class="grid-2">
	<section class="card">
		<header class="card-h">
			<h3>Continue learning</h3>
			<a class="more" href={resolve('/dashboard/courses')}>View all <IconArrowRight size={12} /></a>
		</header>
		<ul class="learn-list">
			{#each continueLearning as l (l.href)}
				<li>
					<a href={resolve(l.href as unknown as '/')} class="learn-row">
						<span class="learn-ic"><IconPlayerPlay size={14} /></span>
						<div class="learn-body">
							<p class="lt">{l.title}</p>
							<ProgressBar value={l.progress} size="sm" />
							<p class="ls">{l.last}</p>
						</div>
						<IconArrowRight size={14} />
					</a>
				</li>
			{/each}
		</ul>
	</section>

	<section class="card">
		<header class="card-h">
			<h3>Upcoming charges</h3>
			<a class="more" href={resolve('/dashboard/billing')}>Billing <IconArrowRight size={12} /></a>
		</header>
		<ul class="charges">
			{#each upcomingCharges as c (c.label)}
				<li>
					<div>
						<p class="ct">{c.label}</p>
						<p class="cd">{c.date}</p>
					</div>
					<strong>{c.amount}</strong>
				</li>
			{/each}
		</ul>
		<p class="card-note">Manage payment methods and invoices from the Billing page.</p>
	</section>
</div>

<section class="card">
	<header class="card-h">
		<h3>Recent downloads</h3>
		<a class="more" href={resolve('/dashboard/downloads')}
			>All downloads <IconArrowRight size={12} /></a
		>
	</header>
	<table class="dl-table">
		<thead>
			<tr><th>File</th><th>Type</th><th>Size</th><th class="right">Date</th><th></th></tr>
		</thead>
		<tbody>
			{#each recentDownloads as d (d.name)}
				<tr>
					<td class="dl-name">{d.name}</td>
					<td><Badge variant="outline" size="sm">{d.kind}</Badge></td>
					<td class="muted">{d.size}</td>
					<td class="right muted">{d.date}</td>
					<td class="right">
						<button class="dl-btn" type="button" aria-label="Download">
							<IconDownload size={14} />
						</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</section>

<style>
	.hero {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: var(--space-4);
		margin-bottom: clamp(2rem, 4vw, 3rem);
		flex-wrap: wrap;
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
	.hero h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: var(--space-2) 0 var(--space-2);
		line-height: var(--leading-tight);
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}
	.hero-actions {
		display: flex;
		gap: var(--space-2);
	}

	.stats {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	@media (min-width: 640px) {
		.stats {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (min-width: 1280px) {
		.stats {
			grid-template-columns: repeat(4, 1fr);
		}
	}

	.grid-2 {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
		margin-bottom: var(--space-4);
	}
	@media (min-width: 1024px) {
		.grid-2 {
			grid-template-columns: 1.4fr 1fr;
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
	.card-note {
		margin: var(--space-4) 0 0;
		font-size: var(--text-xs);
		color: var(--ink-400);
	}

	.learn-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.learn-row {
		display: grid;
		grid-template-columns: 36px 1fr auto;
		gap: var(--space-3);
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		text-decoration: none;
		color: var(--ink-200);
		transition: all var(--dur-base) var(--ease-out);
	}
	.learn-row:hover {
		border-color: var(--border-gold);
		transform: translateX(2px);
		color: var(--ink-100);
	}
	.learn-ic {
		width: 36px;
		height: 36px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-3);
		color: var(--gold-400);
		border-radius: var(--radius-full);
	}
	.learn-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		min-width: 0;
	}
	.lt {
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
	}
	.ls {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
	}

	.charges {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.charges li {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.ct {
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		margin: 0;
		color: var(--ink-100);
	}
	.cd {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 2px 0 0;
	}
	.charges strong {
		font-family: var(--font-display);
		font-size: var(--text-md);
		color: var(--ink-100);
	}

	.dl-table {
		width: 100%;
		border-collapse: collapse;
	}
	.dl-table th {
		text-align: left;
		padding: 0 var(--space-3) var(--space-3);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		border-bottom: 1px solid var(--border-default);
	}
	.dl-table th.right {
		text-align: right;
	}
	.dl-table td {
		padding: var(--space-4) var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
		border-bottom: 1px solid var(--border-subtle);
	}
	.dl-table td.right {
		text-align: right;
	}
	.dl-table tr:last-child td {
		border-bottom: 0;
	}
	.dl-name {
		color: var(--ink-100);
		font-weight: var(--weight-medium);
	}
	.dl-table .muted {
		color: var(--ink-400);
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
