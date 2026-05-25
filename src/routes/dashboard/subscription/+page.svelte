<script lang="ts">
	import {
		IconStar,
		IconCalendar,
		IconReceiptTax,
		IconArrowRight,
		IconAlertTriangle,
		IconPlayerPause,
		IconRefresh,
		IconCheck,
		IconX
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import { DAY_TRADING_PLANS } from '$lib/data/plans.js';
	import { formatPrice } from '$lib/utils/money.js';
	import { toasts } from '$lib/stores/toast.svelte.js';
	import { resolve } from '$app/paths';

	let cancelOpen = $state(false);
	let pauseOpen = $state(false);
	let changeOpen = $state(false);

	let cancelStep = $state<'reason' | 'retention' | 'confirm' | 'done'>('reason');
	let cancelReason = $state('');
	let pauseMonths = $state('1');
	let scheduledCancel = $state(false);

	const currentPlan = DAY_TRADING_PLANS.find((p) => p.cadence === 'quarterly')!;
	const nextRenewalDate = 'Jun 24, 2026';
	const subscriptionStart = 'Nov 18, 2025';
	const usageStats = [
		{ label: 'Live-room hours', value: '124 / mo', delta: '+18%' },
		{ label: 'Alerts triggered', value: '847 / mo', delta: '+6%' },
		{ label: 'Briefings opened', value: '42 / mo', delta: '+12%' }
	];

	function startCancel() {
		cancelOpen = true;
		cancelStep = 'reason';
		cancelReason = '';
	}
	function proceedCancel() {
		if (cancelStep === 'reason') cancelStep = 'retention';
		else if (cancelStep === 'retention') cancelStep = 'confirm';
		else if (cancelStep === 'confirm') {
			scheduledCancel = true;
			cancelStep = 'done';
			toasts.info('Cancellation scheduled', `Access continues through ${nextRenewalDate}.`);
		}
	}
	function reactivate() {
		scheduledCancel = false;
		toasts.success('Subscription reactivated.');
	}
	function applyPause() {
		pauseOpen = false;
		toasts.info(`Subscription paused for ${pauseMonths} month${pauseMonths === '1' ? '' : 's'}.`);
	}
	function applyChange(slug: string) {
		changeOpen = false;
		toasts.success('Plan change scheduled at next renewal.', `Switching to the ${slug} cadence.`);
	}

	const reasons = [
		'Too expensive',
		'Not using it enough',
		'Switching to another service',
		'Took a break from trading',
		'Other'
	];
</script>

<Seo title="Subscription" noindex />

<section class="layout">
	<!-- CURRENT PLAN -->
	<article class="plan-card">
		{#if scheduledCancel}
			<div class="banner banner-warn">
				<IconAlertTriangle size={16} />
				Cancellation scheduled for {nextRenewalDate}. You retain full access until then.
				<button class="banner-action" type="button" onclick={reactivate}>Reactivate</button>
			</div>
		{/if}
		<header class="plan-h">
			<div>
				<p class="eyebrow"><IconStar size={12} />Current plan</p>
				<h2>Day Trading · {currentPlan.name}</h2>
				<p class="muted">Subscription since {subscriptionStart}</p>
			</div>
			<div class="plan-price">
				<span class="amt">{formatPrice(currentPlan.priceCents)}</span>
				<span class="cad">/ quarter</span>
			</div>
		</header>

		<dl class="meta">
			<div>
				<dt>Next renewal</dt>
				<dd><IconCalendar size={14} />{nextRenewalDate}</dd>
			</div>
			<div>
				<dt>Status</dt>
				<dd>
					<Badge variant={scheduledCancel ? 'warning' : 'success'} size="sm"
						>{scheduledCancel ? 'Cancels soon' : 'Active'}</Badge
					>
				</dd>
			</div>
			<div>
				<dt>Payment</dt>
				<dd>•••• 4242 · Visa</dd>
			</div>
		</dl>

		<div class="cta-row">
			<Button variant="primary" size="md" onclick={() => (changeOpen = true)}>Change plan</Button>
			<Button variant="gold-outline" size="md" onclick={() => (pauseOpen = true)}>
				{#snippet iconLeft()}<IconPlayerPause size={14} />{/snippet}
				Pause
			</Button>
			{#if !scheduledCancel}
				<Button variant="ghost" size="md" onclick={startCancel}>Cancel subscription</Button>
			{/if}
		</div>
	</article>

	<!-- USAGE / VALUE -->
	<article class="card">
		<header class="card-h">
			<h3>Usage this month</h3>
			<a class="more" href={resolve('/dashboard')}>Overview <IconArrowRight size={12} /></a>
		</header>
		<div class="usage-grid">
			{#each usageStats as u (u.label)}
				<div class="usage">
					<p class="ul">{u.label}</p>
					<p class="uv">{u.value}</p>
					<span class="ud">{u.delta} vs. last month</span>
				</div>
			{/each}
		</div>
	</article>

	<!-- BENEFITS -->
	<article class="card">
		<header class="card-h">
			<h3>What is included in your tier</h3>
			<Badge variant="gold">Quarterly</Badge>
		</header>
		<ul class="benefits">
			{#each currentPlan.highlights as h (h)}
				<li><span class="ck"><IconCheck size={12} stroke={3} /></span>{h}</li>
			{/each}
		</ul>
	</article>
</section>

<!-- CHANGE PLAN MODAL -->
<Modal
	bind:open={changeOpen}
	title="Change your plan"
	description="Switching is applied at the next renewal — your remaining time is preserved."
	size="lg"
>
	<div class="change-grid">
		{#each DAY_TRADING_PLANS as plan (plan.id)}
			<button
				type="button"
				class="change-card"
				class:is-current={plan.cadence === currentPlan.cadence}
				onclick={() => applyChange(plan.cadence)}
				disabled={plan.cadence === currentPlan.cadence}
			>
				<div class="cc-h">
					<strong>{plan.name}</strong>
					{#if plan.cadence === currentPlan.cadence}<Badge variant="outline" size="sm"
							>Current</Badge
						>{:else if plan.savingsPct}<Badge variant="success" size="sm"
							>Save {plan.savingsPct}%</Badge
						>{/if}
				</div>
				<p class="cc-price">{formatPrice(plan.priceCents)}<span>/ {plan.cadence}</span></p>
				<p class="cc-tag">{plan.tagline}</p>
			</button>
		{/each}
	</div>
</Modal>

<!-- PAUSE MODAL -->
<Modal
	bind:open={pauseOpen}
	title="Pause your subscription"
	description="Pick how long. You will not be charged during the pause and access is suspended."
	size="sm"
>
	<div class="pause">
		<label class="pause-opt">
			<input type="radio" name="pause" value="1" bind:group={pauseMonths} />
			<span><strong>1 month</strong><small>Resume automatically next month</small></span>
		</label>
		<label class="pause-opt">
			<input type="radio" name="pause" value="2" bind:group={pauseMonths} />
			<span><strong>2 months</strong><small>Best for short breaks</small></span>
		</label>
		<label class="pause-opt">
			<input type="radio" name="pause" value="3" bind:group={pauseMonths} />
			<span><strong>3 months</strong><small>Maximum pause window</small></span>
		</label>
	</div>
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (pauseOpen = false)}>Never mind</Button>
		<Button variant="primary" onclick={applyPause}>
			{#snippet iconLeft()}<IconPlayerPause size={14} />{/snippet}
			Confirm pause
		</Button>
	{/snippet}
</Modal>

<!-- CANCEL FLOW MODAL -->
<Modal bind:open={cancelOpen} title="Cancel subscription" size="md">
	{#if cancelStep === 'reason'}
		<p class="muted-p">Help us improve — what is the main reason for cancelling?</p>
		<div class="reasons">
			{#each reasons as r (r)}
				<label class="reason">
					<input type="radio" name="reason" value={r} bind:group={cancelReason} />
					<span>{r}</span>
				</label>
			{/each}
		</div>
	{:else if cancelStep === 'retention'}
		<div class="retention">
			<p class="eyebrow"><IconRefresh size={12} />Wait — one offer</p>
			<h3>Pause instead of cancel.</h3>
			<p>
				Take up to three months off, no charge, keep all your progress. When you come back, your
				plan picks up where you left off.
			</p>
			<div class="retention-actions">
				<Button
					variant="primary"
					onclick={() => {
						cancelOpen = false;
						pauseOpen = true;
					}}>Pause instead</Button
				>
				<button class="muted-link" type="button" onclick={() => (cancelStep = 'confirm')}
					>No thanks — continue cancelling</button
				>
			</div>
		</div>
	{:else if cancelStep === 'confirm'}
		<p class="muted-p">You are about to schedule a cancellation. Here is what happens:</p>
		<ul class="confirm-list">
			<li>
				<span class="ck"><IconCheck size={12} stroke={3} /></span>You keep full access until
				<strong>{nextRenewalDate}</strong>
			</li>
			<li>
				<span class="ck"><IconCheck size={12} stroke={3} /></span>No further charges will be made
			</li>
			<li>
				<span class="ck"><IconCheck size={12} stroke={3} /></span>You can reactivate any time before
				the end date
			</li>
		</ul>
	{:else if cancelStep === 'done'}
		<div class="done">
			<div class="done-ic"><IconCheck size={28} stroke={3} /></div>
			<h3>Cancellation scheduled.</h3>
			<p>
				You will keep access through {nextRenewalDate}. Change of mind? You can reactivate any time
				from this page.
			</p>
		</div>
	{/if}

	{#snippet footer()}
		{#if cancelStep === 'done'}
			<Button variant="primary" onclick={() => (cancelOpen = false)}>Got it</Button>
		{:else if cancelStep !== 'retention'}
			<Button variant="ghost" onclick={() => (cancelOpen = false)}>
				{#snippet iconLeft()}<IconX size={14} />{/snippet}
				Never mind
			</Button>
			<Button
				variant="danger"
				onclick={proceedCancel}
				disabled={cancelStep === 'reason' && !cancelReason}
			>
				{cancelStep === 'confirm' ? 'Confirm cancellation' : 'Continue'}
			</Button>
		{/if}
	{/snippet}
</Modal>

<style>
	.layout {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (--bp-xl) {
		.layout {
			grid-template-columns: 1.6fr 1fr;
		}
		.layout .plan-card {
			grid-column: 1 / -1;
		}
	}

	.plan-card {
		padding: clamp(1.5rem, 3vw, 2.5rem);
		background: linear-gradient(160deg, rgba(232, 182, 96, 0.08), var(--surface-1));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background: var(--warning-bg);
		color: var(--warning);
		border: 1px solid var(--warning);
		border-radius: var(--radius-md);
		font-size: var(--text-sm);
	}
	.banner-action {
		margin-left: auto;
		background: var(--warning);
		color: var(--surface-0);
		padding: 4px 10px;
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		border-radius: var(--radius-full);
		border: 0;
	}

	.plan-h {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-3);
		flex-wrap: wrap;
	}
	.eyebrow {
		display: inline-flex;
		align-items: center;
		gap: 4px;
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
	.plan-h h2 {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		margin: var(--space-2) 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}
	.plan-price {
		font-family: var(--font-display);
		text-align: right;
	}
	.amt {
		font-size: var(--text-4xl);
		font-weight: var(--weight-semibold);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
	}
	.cad {
		font-size: var(--text-sm);
		color: var(--ink-400);
	}

	.meta {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: var(--space-4);
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.meta dt {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.meta dd {
		margin: 4px 0 0;
		font-size: var(--text-sm);
		color: var(--ink-100);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.meta dd :global(svg) {
		color: var(--gold-400);
	}

	.cta-row {
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
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
		gap: var(--space-3);
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

	.usage-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
		gap: var(--space-4);
	}
	.usage {
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.ul {
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.uv {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		color: var(--ink-100);
		margin: var(--space-2) 0 4px;
		line-height: 1;
	}
	.ud {
		font-size: var(--text-2xs);
		color: var(--success);
		font-weight: var(--weight-semibold);
	}

	.benefits {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.benefits li {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.ck {
		display: inline-flex;
		width: 20px;
		height: 20px;
		align-items: center;
		justify-content: center;
		background: rgba(232, 182, 96, 0.18);
		color: var(--gold-300);
		border-radius: var(--radius-full);
	}

	/* Change modal */
	.change-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-3);
	}
	@media (--bp-md) {
		.change-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	.change-card {
		padding: var(--space-5);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		text-align: left;
		cursor: pointer;
		transition: all var(--dur-base) var(--ease-out);
	}
	.change-card:hover:not(:disabled) {
		border-color: var(--border-gold);
		transform: translateY(-2px);
	}
	.change-card.is-current {
		background: var(--surface-3);
		cursor: default;
	}
	.cc-h {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.cc-h strong {
		font-family: var(--font-display);
		font-size: var(--text-md);
		color: var(--ink-100);
	}
	.cc-price {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		color: var(--ink-100);
		margin: var(--space-3) 0 var(--space-2);
	}
	.cc-price span {
		font-size: var(--text-xs);
		color: var(--ink-400);
		font-weight: var(--weight-regular);
	}
	.cc-tag {
		font-size: var(--text-xs);
		color: var(--ink-300);
		margin: 0;
		line-height: var(--leading-relaxed);
	}

	/* Pause */
	.pause {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.pause-opt {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		cursor: pointer;
	}
	.pause-opt:hover {
		border-color: var(--border-gold);
	}
	.pause-opt input {
		accent-color: var(--gold-500);
	}
	.pause-opt strong {
		font-size: var(--text-sm);
		color: var(--ink-100);
		display: block;
	}
	.pause-opt small {
		font-size: var(--text-xs);
		color: var(--ink-400);
		display: block;
		margin-top: 2px;
	}

	/* Cancel flow */
	.muted-p {
		color: var(--ink-300);
		font-size: var(--text-sm);
		margin: 0 0 var(--space-4);
	}
	.reasons {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.reason {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		padding: var(--space-3) var(--space-4);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		cursor: pointer;
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.reason:hover {
		border-color: var(--border-gold);
	}
	.reason input {
		accent-color: var(--gold-500);
	}

	.retention {
		text-align: center;
		padding: var(--space-3) 0;
	}
	.retention h3 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: var(--space-3) 0;
	}
	.retention p {
		color: var(--ink-300);
		max-width: 44ch;
		margin: 0 auto var(--space-5);
		line-height: var(--leading-relaxed);
	}
	.retention-actions {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		align-items: center;
	}
	.muted-link {
		background: transparent;
		color: var(--ink-400);
		font-size: var(--text-xs);
		text-decoration: underline;
		border: 0;
	}
	.muted-link:hover {
		color: var(--ink-200);
	}

	.confirm-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.confirm-list li {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		font-size: var(--text-sm);
		color: var(--ink-200);
	}

	.done {
		text-align: center;
		padding: var(--space-4) 0;
	}
	.done-ic {
		width: 64px;
		height: 64px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		border-radius: var(--radius-full);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		margin-bottom: var(--space-3);
	}
	.done h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0 0 var(--space-3);
	}
	.done p {
		color: var(--ink-300);
		font-size: var(--text-sm);
		margin: 0;
		line-height: var(--leading-relaxed);
	}
</style>
