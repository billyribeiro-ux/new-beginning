<script lang="ts">
	import type { PageData } from './$types';
	import IconStarFilled from '@tabler/icons-svelte/icons/star-filled';
	import IconCheck from '@tabler/icons-svelte/icons/check';
	import IconShieldCheck from '@tabler/icons-svelte/icons/shield-check';
	import IconRefresh from '@tabler/icons-svelte/icons/refresh';
	import IconHeadset from '@tabler/icons-svelte/icons/headset';
	import IconArrowRight from '@tabler/icons-svelte/icons/arrow-right';
	import IconChartCandle from '@tabler/icons-svelte/icons/chart-candle';
	import IconDownload from '@tabler/icons-svelte/icons/download';
	import Seo from '$lib/components/seo/Seo.svelte';
	import JsonLd from '$lib/components/seo/JsonLd.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import FeatureGrid from '$lib/components/marketing/FeatureGrid.svelte';
	import FAQSection from '$lib/components/marketing/FAQSection.svelte';
	import CTABanner from '$lib/components/marketing/CTABanner.svelte';
	import TestimonialCarousel from '$lib/components/marketing/TestimonialCarousel.svelte';
	import PriceTag from '$lib/components/commerce/PriceTag.svelte';
	import AddToCartButton from '$lib/components/commerce/AddToCartButton.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { productLd } from '$lib/utils/jsonld.js';
	import { INDICATOR_FAQS } from '$lib/data/faqs.js';
	import { fadeUp, stagger } from '$lib/animations/attachments.js';

	let { data }: { data: PageData } = $props();
	const p = $derived(data.product);
</script>

<Seo
	title={p.name}
	description={p.tagline}
	type="product"
	keywords={[p.name, p.kind, 'trading indicator', 'NinjaTrader', 'TradingView']}
/>

<JsonLd
	data={productLd({
		name: p.name,
		description: p.tagline,
		priceCents: p.priceCents,
		slug: p.slug,
		rating: p.rating
	})}
/>

<section class="pdp-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Indicators', href: '/indicators' },
				{ label: p.name, href: `/indicators/${p.slug}` }
			]}
		/>

		<div class="hero-grid">
			<div class="left" {@attach fadeUp({ y: 20 })}>
				<div class="meta-row">
					<Badge variant="gold">
						<IconChartCandle size={12} />Indicator
					</Badge>
					{#if p.badge}<Badge variant="outline">{p.badge}</Badge>{/if}
					<div class="rating">
						<IconStarFilled size={14} />
						<span>{p.rating.value.toFixed(1)}</span>
						<span class="muted">· {p.rating.count.toLocaleString()} reviews</span>
					</div>
				</div>
				<h1>{p.name}</h1>
				<p class="tagline">{p.tagline}</p>
				<p class="desc">{p.description}</p>

				<ul class="highlights" {@attach stagger({ stagger: 0.05, y: 12 })}>
					{#each p.highlights as h (h)}
						<li><span class="ck"><IconCheck size={12} stroke={3} /></span>{h}</li>
					{/each}
				</ul>
			</div>

			<aside class="purchase" {@attach fadeUp({ y: 24, delay: 0.15 })}>
				<div class="poster" style:background={p.media.posterColor}>
					<div class="poster-grid"></div>
					<div class="poster-name">
						<span class="eyebrow">Pro Indicator</span>
						<h3>{p.name}</h3>
					</div>
				</div>
				<div class="purchase-body">
					<div class="price-row">
						<PriceTag cents={p.priceCents} originalCents={p.originalPriceCents} size="lg" />
						<span class="lifetime">Lifetime · all updates</span>
					</div>
					<AddToCartButton
						line={{
							id: `prod-${p.id}`,
							kind: 'product',
							slug: p.slug,
							name: p.name,
							subtitle: 'Indicator · Lifetime',
							priceCents: p.priceCents
						}}
						variant="primary"
						size="lg"
						fullWidth
					/>
					<Button variant="gold-outline" size="md" fullWidth href="/free-guide">
						{#snippet iconLeft()}<IconDownload size={14} />{/snippet}
						Try the free Greeks Guide first
					</Button>
					<ul class="trust">
						<li><IconShieldCheck size={16} /> 14-day money-back guarantee</li>
						<li><IconRefresh size={16} /> Lifetime updates</li>
						<li><IconHeadset size={16} /> Priority email + Discord support</li>
					</ul>
				</div>
			</aside>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading
			eyebrow="What you get"
			title="Six pillars, one disciplined engine."
			subtitle="Every feature is a tool you can apply on Monday morning."
		/>
		<div style="margin-top: clamp(2.5rem, 5vw, 4rem);">
			<FeatureGrid features={p.features} columns={3} />
		</div>
	</div>
</section>

<section class="section bg-dark">
	<div class="container">
		<div class="specs-grid">
			<div {@attach fadeUp({ y: 18 })}>
				<p class="eyebrow">Specifications</p>
				<h2>The hard numbers.</h2>
				<p class="lead">Lorem ipsum dolor sit amet. The technical surface area, written plainly.</p>
			</div>
			<dl class="specs">
				{#each p.specs ?? [] as s (s.label)}
					<div class="spec-row">
						<dt>{s.label}</dt>
						<dd>{s.value}</dd>
					</div>
				{/each}
			</dl>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<div class="two-col">
			<div class="col-card" {@attach fadeUp({ y: 18 })}>
				<p class="eyebrow">In the box</p>
				<h3>What ships when you buy.</h3>
				<ul class="ck-list">
					{#each p.deliverables ?? [] as d (d)}
						<li><span class="ck"><IconCheck size={12} stroke={3} /></span>{d}</li>
					{/each}
				</ul>
			</div>
			<div class="col-card" {@attach fadeUp({ y: 18, delay: 0.1 })}>
				<p class="eyebrow">Requirements</p>
				<h3>Before you install.</h3>
				<ul class="ck-list">
					{#each p.requirements ?? [] as r (r)}
						<li><span class="ck"><IconCheck size={12} stroke={3} /></span>{r}</li>
					{/each}
				</ul>
			</div>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="From the desk" title="What members are saying." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<TestimonialCarousel />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="FAQ" title="Common questions." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<FAQSection items={INDICATOR_FAQS} />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<CTABanner
			eyebrow="Ready to deploy"
			title="Install Revolution Ranger in under three minutes."
			subtitle="Lifetime updates, priority support, and a 14-day refund window. Risk-free."
			ctaLabel="Add to cart"
			ctaHref="#"
		/>
	</div>
</section>

<style>
	.pdp-hero {
		padding: clamp(3rem, 6vw, 5rem) 0;
		background: radial-gradient(ellipse at 80% 0%, rgba(232, 182, 96, 0.12), transparent 60%);
	}
	.hero-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		margin-top: var(--space-6);
		align-items: start;
	}
	@media (--bp-lg) {
		.hero-grid {
			grid-template-columns: 1.2fr 1fr;
		}
	}

	.meta-row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		flex-wrap: wrap;
		margin-bottom: var(--space-5);
	}
	.rating {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--gold-400);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
	}
	.rating .muted {
		color: var(--ink-400);
		font-weight: var(--weight-regular);
		margin-inline-start: 4px;
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2.25rem, 4vw, 3.75rem);
		margin: 0 0 var(--space-4);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
	}
	.tagline {
		font-size: var(--text-lg);
		color: var(--ink-200);
		margin: 0 0 var(--space-5);
		line-height: var(--leading-snug);
	}
	.desc {
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		font-size: var(--text-md);
	}
	.highlights {
		list-style: none;
		margin-top: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.highlights li {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.ck {
		display: inline-flex;
		width: 22px;
		height: 22px;
		align-items: center;
		justify-content: center;
		background: rgba(232, 182, 96, 0.18);
		color: var(--gold-300);
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}

	.purchase {
		position: sticky;
		top: calc(var(--navbar-height) + var(--space-4));
		display: flex;
		flex-direction: column;
		background: var(--surface-1);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		overflow: hidden;
		box-shadow: var(--shadow-elev-3), var(--glow-gold);
	}
	.poster {
		aspect-ratio: 5 / 3;
		position: relative;
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		overflow: hidden;
	}
	.poster-grid {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgba(232, 182, 96, 0.08) 1px, transparent 1px),
			linear-gradient(90deg, rgba(232, 182, 96, 0.08) 1px, transparent 1px);
		background-size: 32px 32px;
		mask-image: radial-gradient(ellipse at center, black 30%, transparent 80%);
		-webkit-mask-image: radial-gradient(ellipse at center, black 30%, transparent 80%);
	}
	.poster-name {
		position: relative;
		z-index: 1;
	}
	.poster-name .eyebrow {
		font-size: var(--text-2xs);
		color: var(--gold-300);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		margin: 0;
	}
	.poster-name .eyebrow::before {
		display: none;
	}
	.poster-name h3 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		color: var(--ink-100);
		margin: var(--space-2) 0 0;
	}

	.purchase-body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.price-row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: var(--space-3);
		flex-wrap: wrap;
	}
	.lifetime {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
	}
	.trust {
		list-style: none;
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-default);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		margin-top: var(--space-2);
	}
	.trust li {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-xs);
		color: var(--ink-300);
	}
	.trust li :global(svg) {
		color: var(--gold-400);
		flex-shrink: 0;
	}

	.section {
		padding-block: clamp(4rem, 8vw, 7rem);
	}
	.bg-dark {
		background: linear-gradient(180deg, var(--surface-0), #050507);
		border-block: 1px solid var(--border-subtle);
	}

	.specs-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
	}
	@media (--bp-lg) {
		.specs-grid {
			grid-template-columns: 1fr 1.4fr;
		}
	}
	.specs-grid .eyebrow {
		font-size: var(--text-2xs);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		margin: 0;
	}
	.specs-grid .eyebrow::before {
		display: none;
	}
	.specs-grid h2 {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		margin: var(--space-3) 0 var(--space-3);
	}
	.specs-grid .lead {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
	}

	.specs {
		display: flex;
		flex-direction: column;
		gap: 0;
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		overflow: hidden;
	}
	.spec-row {
		display: grid;
		grid-template-columns: 1fr 1.6fr;
		gap: var(--space-4);
		padding: var(--space-4) var(--space-5);
		border-top: 1px solid var(--border-subtle);
	}
	.spec-row:first-child {
		border-top: 0;
	}
	.spec-row dt {
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.spec-row dd {
		margin: 0;
		font-size: var(--text-sm);
		color: var(--ink-100);
	}

	.two-col {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-6);
	}
	@media (--bp-md) {
		.two-col {
			grid-template-columns: 1fr 1fr;
		}
	}
	.col-card {
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
	}
	.col-card .eyebrow {
		font-size: var(--text-2xs);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		margin: 0;
	}
	.col-card .eyebrow::before {
		display: none;
	}
	.col-card h3 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: var(--space-3) 0 var(--space-5);
	}
	.ck-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.ck-list li {
		display: flex;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
		line-height: var(--leading-snug);
	}
</style>
