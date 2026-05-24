<script lang="ts">
	import { IconBell, IconBellOff, IconMoon, IconCheck, IconDots } from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Switch from '$lib/components/ui/Switch.svelte';
	import Button from '$lib/components/ui/Button.svelte';

	let prefs = $state({
		productUpdates: { email: true, inApp: true },
		billing: { email: true, inApp: true },
		courseProgress: { email: false, inApp: true },
		marketAlerts: { email: true, inApp: true },
		marketing: { email: false, inApp: false }
	});
	let dndEnabled = $state(false);
	let dndStart = $state('21:00');
	let dndEnd = $state('07:00');

	const categories = [
		{
			key: 'productUpdates' as const,
			label: 'Product updates',
			desc: 'New indicators, course modules, feature releases.'
		},
		{
			key: 'billing' as const,
			label: 'Billing',
			desc: 'Invoices, renewals, payment method issues.'
		},
		{
			key: 'courseProgress' as const,
			label: 'Course progress',
			desc: 'Reminders, cohort kickoffs, alumni events.'
		},
		{
			key: 'marketAlerts' as const,
			label: 'Market alerts',
			desc: 'Pre-market briefings and live-room highlights.'
		},
		{
			key: 'marketing' as const,
			label: 'Promotions',
			desc: 'Occasional special offers — never more than monthly.'
		}
	];

	const feed = [
		{
			id: 1,
			title: 'Revolution Ranger v2.4.1 released',
			body: 'Includes refined CL preset and new audio alert tones.',
			when: '2h ago',
			read: false
		},
		{
			id: 2,
			title: 'New cohort opens June 2',
			body: 'Options 101 spring/summer intake is live — reserve a seat.',
			when: '14h ago',
			read: false
		},
		{
			id: 3,
			title: 'Invoice INV-2026-0184 paid',
			body: '$697.00 · Quarterly subscription renewal.',
			when: '3d ago',
			read: true
		},
		{
			id: 4,
			title: 'Weekly review session scheduled',
			body: 'Friday 3pm ET — we are reviewing NQ Wednesday.',
			when: '5d ago',
			read: true
		}
	];

	function markAllRead() {
		for (const item of feed) item.read = true;
	}
</script>

<Seo title="Notifications" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Notifications</p>
		<h2>Choose exactly what you hear.</h2>
		<p class="muted">Granular controls per channel. Quiet hours apply to all.</p>
	</div>
</header>

<div class="grid">
	<article class="card full">
		<header class="card-h"><h3><IconBell size={16} />Channels by category</h3></header>
		<table class="prefs">
			<thead>
				<tr>
					<th>Category</th>
					<th class="center">Email</th>
					<th class="center">In-app</th>
				</tr>
			</thead>
			<tbody>
				{#each categories as cat}
					<tr>
						<td>
							<p class="pl">{cat.label}</p>
							<p class="pd">{cat.desc}</p>
						</td>
						<td class="center"><Switch bind:checked={prefs[cat.key].email} /></td>
						<td class="center"><Switch bind:checked={prefs[cat.key].inApp} /></td>
					</tr>
				{/each}
			</tbody>
		</table>
	</article>

	<article class="card">
		<header class="card-h"><h3><IconMoon size={16} />Do not disturb</h3></header>
		<Switch
			bind:checked={dndEnabled}
			label="Enable quiet hours"
			description="Pause in-app notifications during the window below."
		/>
		<div class="dnd-times" class:is-disabled={!dndEnabled}>
			<label>From <input type="time" bind:value={dndStart} disabled={!dndEnabled} /></label>
			<label>To <input type="time" bind:value={dndEnd} disabled={!dndEnabled} /></label>
		</div>
		<p class="muted">Billing and security notifications always come through.</p>
	</article>

	<article class="card">
		<header class="card-h">
			<h3>Recent</h3>
			<button class="link" type="button" onclick={markAllRead}>Mark all read</button>
		</header>
		<ul class="feed">
			{#each feed as n}
				<li class:unread={!n.read}>
					<span class="dot" aria-hidden="true"></span>
					<div>
						<p class="ft">{n.title}</p>
						<p class="fb">{n.body}</p>
						<p class="fw">{n.when}</p>
					</div>
					<button type="button" aria-label="More" class="more"><IconDots size={14} /></button>
				</li>
			{/each}
		</ul>
	</article>
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

	.grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (min-width: 1280px) {
		.grid {
			grid-template-columns: 1fr 1fr;
		}
		.full {
			grid-column: 1 / -1;
		}
	}

	.card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.card-h {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-3);
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
	.link {
		background: transparent;
		color: var(--gold-300);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-decoration: underline;
		border: 0;
	}

	.prefs {
		width: 100%;
		border-collapse: collapse;
	}
	.prefs th {
		text-align: left;
		padding: 0 var(--space-3) var(--space-3);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		border-bottom: 1px solid var(--border-default);
	}
	.prefs th.center {
		text-align: center;
	}
	.prefs td {
		padding: var(--space-4) var(--space-3);
		border-bottom: 1px solid var(--border-subtle);
		vertical-align: middle;
	}
	.prefs td.center {
		text-align: center;
	}
	.pl {
		font-size: var(--text-sm);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-medium);
	}
	.pd {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
		max-width: 48ch;
	}

	.dnd-times {
		display: flex;
		gap: var(--space-4);
		margin-top: var(--space-2);
	}
	.dnd-times.is-disabled {
		opacity: 0.5;
	}
	.dnd-times label {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.dnd-times input {
		padding: var(--space-2) var(--space-3);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		color: var(--ink-100);
		font-family: inherit;
	}

	.feed {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.feed li {
		display: grid;
		grid-template-columns: 12px 1fr auto;
		gap: var(--space-3);
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
		align-items: flex-start;
	}
	.feed li.unread {
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.04), var(--surface-2));
		border: 1px solid var(--border-gold);
	}
	.feed .dot {
		width: 8px;
		height: 8px;
		background: var(--ink-500);
		border-radius: 50%;
		margin-top: 7px;
	}
	.feed li.unread .dot {
		background: var(--gold-400);
	}
	.ft {
		font-size: var(--text-sm);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.fb {
		font-size: var(--text-xs);
		color: var(--ink-300);
		margin: 4px 0;
		line-height: var(--leading-relaxed);
	}
	.fw {
		font-size: 10px;
		color: var(--ink-500);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		margin: 0;
	}
	.more {
		width: 28px;
		height: 28px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--ink-400);
	}
	.more:hover {
		color: var(--gold-300);
	}
</style>
