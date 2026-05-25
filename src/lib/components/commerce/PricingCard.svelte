<script lang="ts">
	import { IconCheck, IconSparkles } from '@tabler/icons-svelte';
	import PriceTag from './PriceTag.svelte';
	import AddToCartButton from './AddToCartButton.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import type { SubscriptionPlan } from '$lib/data/plans.js';
	import { magnetic } from '$lib/animations/attachments.js';

	type Props = { plan: SubscriptionPlan };
	let { plan }: Props = $props();

	const cadenceLabel = $derived(
		plan.cadence === 'monthly' ? '/ month' : plan.cadence === 'quarterly' ? '/ quarter' : '/ year'
	);
</script>

<article
	class="pricing-card"
	class:is-featured={plan.featured}
	{@attach magnetic({ strength: 0.06 })}
>
	{#if plan.badge}
		<div class="badge-row">
			<Badge variant={plan.featured ? 'gold' : 'outline'} size="sm">
				{#if plan.featured}<IconSparkles size={11} />{/if}
				{plan.badge}
			</Badge>
		</div>
	{/if}

	<header class="head">
		<h3 class="name">{plan.name}</h3>
		<p class="tagline">{plan.tagline}</p>
	</header>

	<div class="price-block">
		<PriceTag cents={plan.priceCents} size="lg" suffix={cadenceLabel} />
		{#if plan.cadence !== 'monthly'}
			<p class="equiv">
				≈ <strong>${(plan.monthlyEquivalentCents / 100).toFixed(0)}</strong>/mo
				{#if plan.savingsPct}<span class="save">· Save {plan.savingsPct}%</span>{/if}
			</p>
		{/if}
	</div>

	<ul class="features">
		{#each plan.highlights as h (h)}
			<li>
				<span class="check"><IconCheck size={14} stroke={3} /></span>
				<span>{h}</span>
			</li>
		{/each}
	</ul>

	<div class="actions">
		<AddToCartButton
			line={{
				id: `plan-${plan.id}`,
				kind: 'plan',
				slug: plan.slug,
				name: `Day Trading · ${plan.name}`,
				subtitle: cadenceLabel.replace('/ ', '').trim(),
				priceCents: plan.priceCents
			}}
			variant={plan.featured ? 'primary' : 'gold-outline'}
			size="lg"
			fullWidth
		/>
		<p class="fineprint">Renews {plan.cadence}. Cancel anytime. 14-day refund.</p>
	</div>
</article>

<style>
	.pricing-card {
		/* Container-relative: the "featured card lifts above siblings"
		 * effect only reads correctly when the card has room. Tying
		 * the lift to its own width (not the viewport) means it
		 * behaves correctly in a 3-up grid AND in a single-column
		 * dashboard sidebar. */
		container-type: inline-size;
		container-name: pricing-card;

		position: relative;
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding: var(--space-8);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-2xl);
		transition:
			transform var(--dur-base) var(--ease-out),
			border-color var(--dur-base) var(--ease-out),
			background var(--dur-base) var(--ease-out);
		overflow: hidden;
	}
	.pricing-card:hover {
		border-color: var(--border-gold);
		transform: translateY(-4px);
	}
	.is-featured {
		background:
			linear-gradient(180deg, rgba(232, 182, 96, 0.06), rgba(23, 23, 28, 0.4)), var(--surface-2);
		border-color: var(--gold-500);
		box-shadow: var(--glow-gold-strong);
		transform: scale(1.02);
	}
	/* Disable the featured-card lift when the card itself is narrow,
	 * regardless of viewport. Stacked single-column layouts always
	 * see flat cards even on a 4K screen. */
	@container pricing-card (max-width: 28rem) {
		.is-featured {
			transform: none;
		}
	}
	.is-featured::before {
		content: '';
		position: absolute;
		inset: -40% -10% auto -10%;
		height: 80%;
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.22), transparent 70%);
		pointer-events: none;
		z-index: 0;
	}

	.badge-row,
	.head,
	.price-block,
	.features,
	.actions {
		position: relative;
		z-index: 1;
	}

	.head {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.name {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.is-featured .name {
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
	}
	.tagline {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}

	.price-block {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		padding-block: var(--space-2);
	}
	.equiv {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
	}
	.equiv strong {
		color: var(--ink-200);
	}
	.save {
		color: var(--success);
		font-weight: var(--weight-semibold);
		margin-left: var(--space-1);
	}

	.features {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding-block: var(--space-4);
		border-block: 1px dashed var(--border-default);
	}
	.features li {
		display: grid;
		grid-template-columns: 20px 1fr;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
		line-height: var(--leading-snug);
	}
	.check {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		background: rgba(232, 182, 96, 0.14);
		color: var(--gold-300);
		border-radius: var(--radius-full);
		flex-shrink: 0;
		margin-top: 1px;
	}

	.actions {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.fineprint {
		text-align: center;
		font-size: var(--text-2xs);
		color: var(--ink-400);
		margin: 0;
	}
</style>
