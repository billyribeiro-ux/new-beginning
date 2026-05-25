<script lang="ts">
	import type { PageData } from './$types';
	import {
		IconStarFilled,
		IconCheck,
		IconCertificate,
		IconClock,
		IconCalendarStats,
		IconPlayerPlayFilled,
		IconLock,
		IconBook2,
		IconShieldCheck
	} from '@tabler/icons-svelte';
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
	import { courseLd } from '$lib/utils/jsonld.js';
	import { COURSE_FAQS } from '$lib/data/faqs.js';
	import { fadeUp, stagger } from '$lib/animations/attachments.js';

	let { data }: { data: PageData } = $props();
	const p = $derived(data.product);

	const curriculum = [
		{ module: 'Module 01', title: 'Options Mechanics', lessons: 4, duration: '52 min' },
		{
			module: 'Module 02',
			title: 'Pricing & The Black-Scholes Intuition',
			lessons: 4,
			duration: '58 min'
		},
		{ module: 'Module 03', title: 'Delta', lessons: 4, duration: '46 min' },
		{ module: 'Module 04', title: 'Gamma & Convexity', lessons: 4, duration: '49 min' },
		{ module: 'Module 05', title: 'Theta & Time Decay', lessons: 4, duration: '44 min' },
		{ module: 'Module 06', title: 'Vega & Volatility', lessons: 4, duration: '61 min' },
		{ module: 'Module 07', title: 'Single-leg Strategies', lessons: 4, duration: '57 min' },
		{ module: 'Module 08', title: 'Verticals & Calendars', lessons: 4, duration: '63 min' },
		{ module: 'Module 09', title: 'Iron Condors & Butterflies', lessons: 4, duration: '68 min' },
		{ module: 'Module 10', title: 'Position Sizing & Risk', lessons: 4, duration: '51 min' },
		{ module: 'Module 11', title: 'Earnings & Event Trades', lessons: 4, duration: '55 min' },
		{ module: 'Module 12', title: 'Capstone Case Studies', lessons: 4, duration: '72 min' }
	];
</script>

<Seo title={p.name} description={p.tagline} type="article" />
<JsonLd
	data={courseLd({ name: p.name, description: p.tagline, slug: p.slug, priceCents: p.priceCents })}
/>

<section class="pdp-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Courses', href: '/courses' },
				{ label: p.name, href: `/courses/${p.slug}` }
			]}
		/>

		<div class="hero-grid">
			<div class="left" {@attach fadeUp({ y: 20 })}>
				<div class="meta-row">
					<Badge variant="gold"><IconBook2 size={12} />Course</Badge>
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
				<div class="quick-meta">
					<div><IconClock size={16} /><span>~12 hours total</span></div>
					<div><IconCalendarStats size={16} /><span>12 modules · 48 lessons</span></div>
					<div><IconCertificate size={16} /><span>Certificate on completion</span></div>
				</div>
			</div>

			<aside class="purchase" {@attach fadeUp({ y: 24, delay: 0.15 })}>
				<div class="poster" style:background={p.media.posterColor}>
					<div class="poster-grid"></div>
					<div class="poster-name">
						<span class="eyebrow">Cohort course</span>
						<h3>{p.name}</h3>
					</div>
					<div class="play"><IconPlayerPlayFilled size={32} /></div>
				</div>
				<div class="purchase-body">
					<div class="price-row">
						<PriceTag cents={p.priceCents} size="lg" />
						<span class="lifetime">Lifetime · all updates</span>
					</div>
					<AddToCartButton
						line={{
							id: `prod-${p.id}`,
							kind: 'product',
							slug: p.slug,
							name: p.name,
							subtitle: 'Course · Lifetime',
							priceCents: p.priceCents
						}}
						variant="primary"
						size="lg"
						fullWidth
					/>
					<ul class="trust">
						<li><IconShieldCheck size={16} /> 14-day refund window</li>
						<li><IconCertificate size={16} /> Signed certificate on completion</li>
						<li><IconCalendarStats size={16} /> Monthly cohort kickoffs + alumni Discord</li>
					</ul>
				</div>
			</aside>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading
			eyebrow="What you will learn"
			title="Six pillars to engineering a clean options book."
		/>
		<div style="margin-top: clamp(2.5rem, 5vw, 4rem);">
			<FeatureGrid features={p.features} columns={3} />
		</div>
	</div>
</section>

<section class="section bg-dark">
	<div class="container">
		<SectionHeading
			eyebrow="Curriculum"
			title="Twelve modules. Forty-eight lessons. Zero hand-waving."
			subtitle="A structured arc from mechanics to live capstones. Self-paced, but anchored by cohort kickoffs."
		/>

		<ol class="curriculum" {@attach stagger({ stagger: 0.04, y: 14 })}>
			{#each curriculum as m, i (m.module)}
				<li class="cu-row" class:is-locked={i > 1}>
					<span class="cu-num">{m.module}</span>
					<div class="cu-body">
						<h3>{m.title}</h3>
						<p>{m.lessons} lessons · {m.duration}</p>
					</div>
					<span class="cu-icon">
						{#if i > 1}<IconLock size={16} />{:else}<IconPlayerPlayFilled size={16} />{/if}
					</span>
				</li>
			{/each}
		</ol>
		<p class="cu-note">
			Lessons 1–8 are unlocked at purchase. Subsequent modules unlock as cohort weeks advance —
			keeping the desk in sync.
		</p>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="Outcomes" title="What graduates take away." align="center" />
		<div class="outcomes" {@attach stagger({ stagger: 0.08, y: 18 })}>
			{#each p.deliverables ?? [] as d (d)}
				<div class="outcome">
					<span class="ck"><IconCheck size={14} stroke={3} /></span>
					<p>{d}</p>
				</div>
			{/each}
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="Alumni voices" title="From the cohort." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<TestimonialCarousel />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<SectionHeading eyebrow="FAQ" title="Common questions." align="center" />
		<div style="margin-top: clamp(2rem, 5vw, 3rem);">
			<FAQSection items={COURSE_FAQS} />
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<CTABanner
			eyebrow="Next cohort kicks off soon"
			title="Engineer the options book you actually want to run."
			subtitle="Lifetime access · cohort kickoff · 14-day refund. Reserve your seat today."
			ctaLabel="Enroll in Options 101"
			ctaHref="#"
		/>
	</div>
</section>

<style>
	.pdp-hero {
		padding: clamp(3rem, 6vw, 5rem) 0;
		background: radial-gradient(ellipse at 20% 0%, rgba(232, 182, 96, 0.12), transparent 60%);
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
	.quick-meta {
		margin-top: var(--space-6);
		display: flex;
		gap: var(--space-5);
		flex-wrap: wrap;
	}
	.quick-meta div {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-sm);
		color: var(--ink-300);
	}
	.quick-meta :global(svg) {
		color: var(--gold-400);
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
	.play {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--gold-300);
		opacity: 0.85;
		pointer-events: none;
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

	.curriculum {
		list-style: none;
		margin-top: clamp(2rem, 5vw, 3rem);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		counter-reset: cu;
	}
	.cu-row {
		display: grid;
		grid-template-columns: 120px 1fr auto;
		gap: var(--space-5);
		align-items: center;
		padding: var(--space-5) var(--space-6);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		transition: all var(--dur-base) var(--ease-out);
	}
	.cu-row:hover {
		border-color: var(--border-gold);
	}
	.cu-row.is-locked {
		opacity: 0.62;
	}
	.cu-num {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
	}
	.cu-body h3 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.cu-body p {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
	.cu-icon {
		width: 36px;
		height: 36px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--gold-400);
	}
	.is-locked .cu-icon {
		color: var(--ink-400);
	}
	.cu-note {
		margin-top: var(--space-5);
		text-align: center;
		font-size: var(--text-xs);
		color: var(--ink-400);
	}

	.outcomes {
		margin-top: clamp(2rem, 5vw, 3rem);
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-3);
	}
	@media (--bp-md) {
		.outcomes {
			grid-template-columns: 1fr 1fr;
		}
	}
	.outcome {
		display: grid;
		grid-template-columns: 28px 1fr;
		gap: var(--space-3);
		padding: var(--space-5);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.outcome p {
		margin: 0;
		font-size: var(--text-sm);
		color: var(--ink-200);
		line-height: var(--leading-relaxed);
	}
	.outcome .ck {
		width: 28px;
		height: 28px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: rgba(232, 182, 96, 0.18);
		color: var(--gold-300);
		border-radius: var(--radius-full);
	}
</style>
