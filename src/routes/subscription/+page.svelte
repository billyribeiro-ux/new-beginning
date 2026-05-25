<script lang="ts">
	import IconCheck from '@tabler/icons-svelte/icons/check';
	import IconShieldCheck from '@tabler/icons-svelte/icons/shield-check';
	import IconUsersGroup from '@tabler/icons-svelte/icons/users-group';
	import IconStarFilled from '@tabler/icons-svelte/icons/star-filled';
	import IconCalendarStats from '@tabler/icons-svelte/icons/calendar-stats';
	import IconHeadset from '@tabler/icons-svelte/icons/headset';
	import Seo from '$lib/components/seo/Seo.svelte';
	import JsonLd from '$lib/components/seo/JsonLd.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import FAQSection from '$lib/components/marketing/FAQSection.svelte';
	import CTABanner from '$lib/components/marketing/CTABanner.svelte';
	import TestimonialCarousel from '$lib/components/marketing/TestimonialCarousel.svelte';
	import PricingCard from '$lib/components/commerce/PricingCard.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { DAY_TRADING_PLANS } from '$lib/data/plans.js';
	import { serviceLd } from '$lib/utils/jsonld.js';
	import { SUBSCRIPTION_FAQS } from '$lib/data/faqs.js';
	import { fadeUp, stagger } from '$lib/animations/attachments.js';

	const compareRows = [
		{ feature: 'Daily pre-market briefings', monthly: true, quarterly: true, annual: true },
		{ feature: 'Live trading room (9:30am–12pm ET)', monthly: true, quarterly: true, annual: true },
		{ feature: 'Real-time alerts · ES, NQ, CL, GC', monthly: true, quarterly: true, annual: true },
		{ feature: 'Weekly trade review session', monthly: true, quarterly: true, annual: true },
		{ feature: 'Quarterly 1:1 strategy review', monthly: false, quarterly: true, annual: true },
		{ feature: 'Macro deep-dive call', monthly: false, quarterly: true, annual: true },
		{ feature: 'Beta indicators access', monthly: false, quarterly: true, annual: true },
		{ feature: 'Revolution Ranger indicator', monthly: false, quarterly: false, annual: true },
		{ feature: 'Options 101 course', monthly: false, quarterly: false, annual: true },
		{ feature: 'Founders Circle Discord', monthly: false, quarterly: false, annual: true }
	];
</script>

<Seo
	title="Day Trading Subscription"
	description="Live day-trading desk, daily briefings, real-time alerts on ES/NQ/CL/GC. Monthly, quarterly, and annual tiers."
	keywords={['day trading subscription', 'trading desk', 'live trading room', 'TradeFlex Trading']}
/>

<JsonLd
	data={serviceLd(
		DAY_TRADING_PLANS.map((p) => ({ name: p.name, priceCents: p.priceCents, cadence: p.cadence }))
	)}
/>

<section class="hero-section">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Subscription', href: '/subscription' }
			]}
		/>
		<div class="hero-grid" {@attach fadeUp({ y: 20 })}>
			<div>
				<Badge variant="gold">
					<IconStarFilled size={12} />Day Trading desk
				</Badge>
				<h1>The desk you would build, if you had the time.</h1>
				<p class="lead">
					Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pre-market briefings, a live
					trading room from the bell to lunch, real-time alerts, and a weekly review cadence that
					compounds. One subscription. Three commitment levels.
				</p>
			</div>
			<aside class="hero-perks">
				<div class="perk">
					<IconShieldCheck size={18} /><span>14-day refund, no contracts</span>
				</div>
				<div class="perk"><IconUsersGroup size={18} /><span>14,200+ active members</span></div>
				<div class="perk">
					<IconCalendarStats size={18} /><span>Live 9:30am–12pm ET, Mon–Fri</span>
				</div>
				<div class="perk">
					<IconHeadset size={18} /><span>Priority email + Discord support</span>
				</div>
			</aside>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<div class="pricing-grid" {@attach stagger({ stagger: 0.12, y: 28 })}>
			{#each DAY_TRADING_PLANS as plan (plan.id)}
				<PricingCard {plan} />
			{/each}
		</div>
		<p class="fineprint">
			All prices in USD. Renew automatically — cancel anytime from your dashboard.
		</p>
	</div>
</section>

<section class="section bg-dark">
	<div class="container">
		<SectionHeading
			eyebrow="Compare tiers"
			title="What is included in each plan."
			subtitle="Apples-to-apples. Pick the cadence that matches your commitment."
		/>

		<div class="compare-wrap">
			<table class="compare">
				<thead>
					<tr>
						<th class="feature-th">Feature</th>
						<th>Monthly</th>
						<th class="featured-th">Quarterly</th>
						<th>Annual</th>
					</tr>
				</thead>
				<tbody>
					{#each compareRows as row (row.feature)}
						<tr>
							<td class="feature-cell">{row.feature}</td>
							<td
								>{#if row.monthly}<span class="ck"><IconCheck size={14} stroke={3} /></span
									>{:else}<span class="muted">—</span>{/if}</td
							>
							<td
								>{#if row.quarterly}<span class="ck"><IconCheck size={14} stroke={3} /></span
									>{:else}<span class="muted">—</span>{/if}</td
							>
							<td
								>{#if row.annual}<span class="ck"><IconCheck size={14} stroke={3} /></span
									>{:else}<span class="muted">—</span>{/if}</td
							>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="Real members" title="What our desk says." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<TestimonialCarousel />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="FAQ" title="Subscription questions, answered." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<FAQSection items={SUBSCRIPTION_FAQS} />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<CTABanner
			eyebrow="The room opens at 9:30am ET"
			title="Stop trading alone. Join the desk."
			subtitle="Cancel anytime. 14-day refund. No retention calls — just process."
			ctaLabel="Start with Quarterly"
			ctaHref="#"
			secondaryLabel="Have questions?"
			secondaryHref="/contact"
		/>
	</div>
</section>

<style>
	.hero-section {
		padding: clamp(4rem, 8vw, 6rem) 0;
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.12), transparent 60%);
	}
	.hero-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		margin-top: var(--space-6);
		align-items: end;
	}
	@media (--bp-lg) {
		.hero-grid {
			grid-template-columns: 1.4fr 1fr;
		}
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2.5rem, 5vw, 4.5rem);
		margin: var(--space-5) 0 var(--space-4);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
	}
	.lead {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		max-width: 60ch;
	}
	.hero-perks {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
	}
	.perk {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.perk :global(svg) {
		color: var(--gold-400);
		flex-shrink: 0;
	}

	.section {
		padding-block: clamp(4rem, 8vw, 8rem);
	}
	.bg-dark {
		background: linear-gradient(180deg, var(--surface-0), #050507);
		border-block: 1px solid var(--border-subtle);
	}

	.pricing-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-6);
		align-items: stretch;
	}
	@media (--bp-md) {
		.pricing-grid {
			grid-template-columns: 1fr 1fr;
		}
	}
	@media (--bp-lg) {
		.pricing-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	.fineprint {
		text-align: center;
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin-top: var(--space-6);
	}

	.compare-wrap {
		margin-top: clamp(2rem, 5vw, 3rem);
		overflow-x: auto;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
	}
	.compare {
		width: 100%;
		border-collapse: collapse;
		min-width: 720px;
	}
	.compare thead {
		background: var(--surface-2);
	}
	.compare th {
		padding: var(--space-5) var(--space-4);
		font-family: var(--font-display);
		font-size: var(--text-md);
		color: var(--ink-100);
		text-align: center;
		font-weight: var(--weight-semibold);
	}
	.compare .feature-th {
		text-align: left;
	}
	.compare .featured-th {
		background: linear-gradient(180deg, rgba(232, 182, 96, 0.18), rgba(232, 182, 96, 0.04));
		color: var(--gold-300);
	}
	.compare td {
		padding: var(--space-4) var(--space-4);
		border-top: 1px solid var(--border-subtle);
		text-align: center;
		font-size: var(--text-sm);
		color: var(--ink-200);
		vertical-align: middle;
	}
	.compare .feature-cell {
		text-align: left;
		color: var(--ink-100);
		font-weight: var(--weight-medium);
	}
	.ck {
		display: inline-flex;
		width: 24px;
		height: 24px;
		align-items: center;
		justify-content: center;
		background: rgba(232, 182, 96, 0.18);
		color: var(--gold-300);
		border-radius: var(--radius-full);
	}
	.muted {
		color: var(--ink-500);
	}
</style>
