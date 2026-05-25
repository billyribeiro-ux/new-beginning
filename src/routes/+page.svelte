<script lang="ts">
	import {
		IconArrowRight,
		IconBolt,
		IconShieldCheck,
		IconCalendarStats,
		IconChartCandle,
		IconBook2,
		IconUsersGroup,
		IconStarFilled,
		IconCheck
	} from '@tabler/icons-svelte';
	import { resolve } from '$app/paths';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Hero from '$lib/components/marketing/Hero.svelte';
	import TrustBar from '$lib/components/marketing/TrustBar.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import FeatureGrid from '$lib/components/marketing/FeatureGrid.svelte';
	import StatBlock from '$lib/components/marketing/StatBlock.svelte';
	import TestimonialCarousel from '$lib/components/marketing/TestimonialCarousel.svelte';
	import FAQSection from '$lib/components/marketing/FAQSection.svelte';
	import CTABanner from '$lib/components/marketing/CTABanner.svelte';
	import ProductCard from '$lib/components/commerce/ProductCard.svelte';
	import PricingCard from '$lib/components/commerce/PricingCard.svelte';
	import EmailCaptureForm from '$lib/components/forms/EmailCaptureForm.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { INDICATORS, COURSES } from '$lib/data/products.js';
	import { DAY_TRADING_PLANS } from '$lib/data/plans.js';
	import { GENERAL_FAQS } from '$lib/data/faqs.js';
	import { fadeUp, stagger } from '$lib/animations/attachments.js';

	const featuredIndicator = INDICATORS[0]!;
	const featuredCourse = COURSES[0]!;

	const pillarFeatures = [
		{
			icon: 'IconChartCandle',
			title: 'Real systems, not signals',
			description:
				'Lorem ipsum dolor sit amet, consectetur adipiscing elit. We give you tooling and an opinionated process — not a stream of trades to mirror.'
		},
		{
			icon: 'IconBolt',
			title: 'Sub-millisecond alerts',
			description:
				'Sed do eiusmod tempor. Our DOM hooks deliver before the next candle prints. No repaint, ever.'
		},
		{
			icon: 'IconShieldCheck',
			title: '14-day money back',
			description:
				'Ut enim ad minim veniam. Try anything for two weeks. If it does not fit your process, we will refund — no questions.'
		},
		{
			icon: 'IconCalendarStats',
			title: 'Cohorted education',
			description:
				'Duis aute irure dolor. Courses run on cohorts with weekly office hours. Self-paced, but never alone.'
		},
		{
			icon: 'IconUsersGroup',
			title: 'Live trading room',
			description:
				'Excepteur sint occaecat cupidatat. Daily 9:30am ET open with seasoned operators on ES · NQ · CL · GC.'
		},
		{
			icon: 'IconBook2',
			title: 'Original research',
			description:
				'Sunt in culpa qui officia. Weekly market structure notes and macro deep-dives written by the desk.'
		}
	];

	const proofStats = [
		{ value: 14200, label: 'Active members', suffix: '+' },
		{
			value: 1.4,
			label: 'Million signals delivered',
			formatter: (n: number) => n.toFixed(1) + 'M'
		},
		{ value: 4.9, label: 'Average rating', formatter: (n: number) => n.toFixed(1) + '/5' },
		{ value: 99.97, label: 'Platform uptime', formatter: (n: number) => n.toFixed(2) + '%' }
	];

	const playbookSteps = [
		{
			n: '01',
			title: 'Plan with structure',
			body: 'Lorem ipsum dolor sit amet. Pre-market briefings frame the day in volatility, levels, and catalysts so you never freelance into noise.'
		},
		{
			n: '02',
			title: 'Execute with conviction',
			body: 'Consectetur adipiscing elit. Tooling like Revolution Ranger keeps you anchored to the regime — not the candle.'
		},
		{
			n: '03',
			title: 'Review without ego',
			body: 'Sed do eiusmod tempor. Weekly cohort reviews force you to confront your tape and tighten the loop.'
		}
	];
</script>

<Seo
	description="Engineered indicators, education, and a live day-trading desk for serious operators. Built by traders who run a real process."
	keywords={[
		'day trading',
		'options trading',
		'trading indicators',
		'trading course',
		'TradeFlex Trading',
		'Options 101',
		'Revolution Ranger'
	]}
/>

<Hero />

<TrustBar />

<!-- VALUE PILLARS -->
<section class="section">
	<div class="container">
		<SectionHeading
			eyebrow="Why TradeFlex"
			title="Engineering, not entertainment."
			subtitle="Everything we ship is built for a single purpose: making your trading process repeatable, measurable, and durable across regimes."
			align="center"
		/>
		<div class="features-wrap">
			<FeatureGrid features={pillarFeatures} columns={3} />
		</div>
	</div>
</section>

<!-- FEATURED PRODUCTS -->
<section class="section bg-dark">
	<div class="container">
		<SectionHeading
			eyebrow="What we ship"
			title="Three opinionated products. One unified process."
			subtitle="Stack them, or take just one — the live desk, the institutional-grade indicator, and the foundational options course."
		/>

		<div class="featured-grid" {@attach stagger({ stagger: 0.1, y: 28 })}>
			<ProductCard product={featuredIndicator} />
			<div class="featured-plan-card">
				{#each DAY_TRADING_PLANS.filter((p) => p.featured) as plan (plan.id)}
					<PricingCard {plan} />
				{/each}
			</div>
			<ProductCard product={featuredCourse} />
		</div>
	</div>
</section>

<!-- PROOF / STATS -->
<section class="section">
	<div class="container">
		<SectionHeading
			eyebrow="Numbers, not narrative"
			title="A desk built on consistency."
			align="center"
		/>
		<div class="stats-wrap">
			<StatBlock stats={proofStats} />
		</div>
	</div>
</section>

<!-- PLAYBOOK -->
<section class="section playbook-section">
	<div class="container">
		<div class="playbook-grid">
			<div {@attach fadeUp({ y: 18 })}>
				<p class="eyebrow">Our playbook</p>
				<h2 class="text-balance">Plan. Execute. Review. Repeat — with a process that scales.</h2>
				<p class="lead text-pretty">
					Lorem ipsum dolor sit amet, consectetur adipiscing elit. The TradeFlex method is a
					three-beat operating cadence used by every member of our desk. No discretion without
					structure. No reviews without evidence.
				</p>
				<div class="cta-row">
					<Button variant="primary" size="lg" href="/about">
						How we trade
						{#snippet iconRight()}<IconArrowRight size={16} />{/snippet}
					</Button>
					<Button variant="gold-outline" size="lg" href="/subscription">See the subscription</Button
					>
				</div>
			</div>

			<ol class="steps" {@attach stagger({ stagger: 0.1, y: 24 })}>
				{#each playbookSteps as s (s.n)}
					<li class="step">
						<span class="num">{s.n}</span>
						<div>
							<h3>{s.title}</h3>
							<p>{s.body}</p>
						</div>
					</li>
				{/each}
			</ol>
		</div>
	</div>
</section>

<!-- LEAD MAGNET -->
<section class="section lead-magnet-section">
	<div class="container">
		<div class="lm-grid" {@attach fadeUp({ y: 24 })}>
			<div class="lm-copy">
				<Badge variant="gold">
					<IconStarFilled size={12} />Free · 24 pages
				</Badge>
				<h2 class="text-balance">The Options Greeks Guide we wish we had on day one.</h2>
				<p class="text-pretty">
					Delta, gamma, theta, vega, rho — explained visually with scenarios you can rebuild in your
					platform of choice. A foundational PDF, free for our community.
				</p>
				<ul class="lm-list">
					<li>
						<span class="check"><IconCheck size={12} stroke={3} /></span>Visual treatment of each
						greek
					</li>
					<li>
						<span class="check"><IconCheck size={12} stroke={3} /></span>Worked examples on SPY,
						AAPL, NVDA
					</li>
					<li>
						<span class="check"><IconCheck size={12} stroke={3} /></span>Position sizing &
						assignment risk
					</li>
					<li>
						<span class="check"><IconCheck size={12} stroke={3} /></span>No upsell — just the guide
					</li>
				</ul>
			</div>
			<div class="lm-form-wrap">
				<div class="lm-form-card">
					<h3>Get the free guide</h3>
					<p>Enter your email — we deliver instantly. No spam, ever.</p>
					<EmailCaptureForm
						source="home-leadmagnet"
						ctaLabel="Send the guide"
						successMessage="Sent. Open your inbox — the PDF link is inside."
					/>
					<p class="fineprint">
						By subscribing you accept our <a href={resolve('/legal/privacy')}>Privacy Policy</a>.
					</p>
				</div>
			</div>
		</div>
	</div>
</section>

<!-- TESTIMONIALS -->
<section class="section">
	<div class="container">
		<SectionHeading
			eyebrow="What members say"
			title="From the people who run the tape."
			subtitle="Verified members across futures, options, and equities desks."
			align="center"
		/>
		<div class="testi-wrap">
			<TestimonialCarousel />
		</div>
	</div>
</section>

<!-- FAQ -->
<section class="section">
	<div class="container">
		<SectionHeading eyebrow="Frequently asked" title="Questions, answered." align="center" />
		<div class="faq-wrap">
			<FAQSection items={GENERAL_FAQS} />
		</div>
	</div>
</section>

<!-- BOTTOM CTA -->
<section class="section">
	<div class="container">
		<CTABanner
			eyebrow="Start with the desk"
			title="The most expensive trade is the one you never reviewed."
			subtitle="Join 14,200+ traders who refuse to wing it. Two-week refund window. Cancel anytime. No contracts."
			ctaLabel="Join the Day Trading desk"
			ctaHref="/subscription"
			secondaryLabel="Browse indicators"
			secondaryHref="/indicators"
		/>
	</div>
</section>

<style>
	.section {
		padding-block: clamp(4rem, 8vw, 8rem);
		position: relative;
	}
	.bg-dark {
		background: linear-gradient(180deg, var(--surface-0), #050507);
	}
	.features-wrap {
		margin-top: clamp(2.5rem, 5vw, 4rem);
	}
	.stats-wrap {
		margin-top: clamp(2rem, 5vw, 4rem);
	}
	.testi-wrap,
	.faq-wrap {
		margin-top: clamp(2.5rem, 5vw, 4rem);
	}

	.featured-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-6);
		margin-top: clamp(2.5rem, 5vw, 4rem);
		align-items: stretch;
	}
	@media (--bp-md) {
		.featured-grid {
			grid-template-columns: 1fr 1fr;
		}
	}
	@media (--bp-xl) {
		.featured-grid {
			grid-template-columns: 1fr 1.05fr 1fr;
		}
	}
	.featured-plan-card {
		display: flex;
	}
	.featured-plan-card :global(.pricing-card) {
		flex: 1;
	}

	.playbook-section {
		background: var(--surface-1);
		border-block: 1px solid var(--border-subtle);
	}
	.playbook-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		align-items: center;
	}
	@media (--bp-lg) {
		.playbook-grid {
			grid-template-columns: 1fr 1fr;
		}
	}

	.playbook-grid h2 {
		font-size: var(--text-4xl);
		margin: var(--space-3) 0 var(--space-5);
	}
	.eyebrow {
		font-size: var(--text-2xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
	}
	.eyebrow::before {
		display: none;
	}
	.lead {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
	}
	.cta-row {
		margin-top: var(--space-6);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.steps {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.step {
		display: grid;
		grid-template-columns: 64px 1fr;
		gap: var(--space-5);
		padding: var(--space-6);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		transition: all var(--dur-base) var(--ease-out);
	}
	.step:hover {
		border-color: var(--border-gold);
		transform: translateX(4px);
	}
	.num {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		font-weight: var(--weight-semibold);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		line-height: 1;
		letter-spacing: var(--tracking-tighter);
	}
	.step h3 {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin: 4px 0 var(--space-2);
	}
	.step p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		margin: 0;
		line-height: var(--leading-relaxed);
	}

	.lead-magnet-section {
		background: var(--surface-0);
	}
	.lm-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		align-items: center;
		padding: clamp(2.5rem, 6vw, 5rem);
		background: linear-gradient(160deg, var(--surface-1), var(--surface-2));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		box-shadow: var(--shadow-elev-3);
		position: relative;
		overflow: hidden;
	}
	.lm-grid::before {
		content: '';
		position: absolute;
		inset: -40% -10% auto -10%;
		height: 80%;
		background: radial-gradient(ellipse at top, rgba(232, 182, 96, 0.18), transparent 60%);
		pointer-events: none;
	}
	@media (--bp-lg) {
		.lm-grid {
			grid-template-columns: 1.2fr 1fr;
		}
	}
	.lm-copy h2 {
		font-family: var(--font-display);
		font-size: var(--text-4xl);
		margin: var(--space-3) 0 var(--space-4);
		line-height: var(--leading-tight);
	}
	.lm-copy p {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
	}
	.lm-list {
		list-style: none;
		margin-top: var(--space-5);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.lm-list li {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.check {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		background: rgba(232, 182, 96, 0.18);
		color: var(--gold-300);
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}

	.lm-form-card {
		padding: var(--space-7);
		/* Opaque fallback for browsers without backdrop-filter. */
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		position: relative;
	}
	@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)) {
		.lm-form-card {
			background: rgba(10, 10, 11, 0.6);
			backdrop-filter: blur(12px);
			-webkit-backdrop-filter: blur(12px);
		}
	}
	.lm-form-card h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0 0 var(--space-2);
	}
	.lm-form-card p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		margin: 0 0 var(--space-5);
	}
	.fineprint {
		margin-top: var(--space-4) !important;
		font-size: var(--text-2xs) !important;
		color: var(--ink-400) !important;
	}
	.fineprint a {
		color: var(--gold-300);
		text-decoration: underline;
	}
</style>
