<script lang="ts">
	import Seo from '$lib/components/seo/Seo.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import ProductCard from '$lib/components/commerce/ProductCard.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { COURSES } from '$lib/data/products.js';
	import { stagger, fadeUp } from '$lib/animations/attachments.js';
	import { IconBook2, IconRocket, IconArrowRight } from '@tabler/icons-svelte';

	type RoadmapEntry = {
		title: string;
		tagline: string;
		quarter: string;
		posterColor: string;
	};

	const ROADMAP: RoadmapEntry[] = [
		{
			title: 'Futures 201: Market Structure',
			tagline:
				'Order flow, market profile, and intraday structure for ES/NQ/CL specialists. Deep, dense, and cohort-led.',
			quarter: 'Q3 2026',
			posterColor: 'linear-gradient(135deg, #20180a, #0a0a0b)'
		},
		{
			title: 'Volatility Trader Bootcamp',
			tagline:
				'From IV term structure to gamma scalping. A focused, intensive course for the systematic vol seller.',
			quarter: 'Q4 2026',
			posterColor: 'linear-gradient(135deg, #1a1322, #0a0a0b)'
		}
	];
</script>

<Seo
	title="Courses"
	description="Structured, cohort-driven trading courses. Visual-first, exam-graded, and built by traders who teach for a living."
	keywords={['trading course', 'options course', 'Options 101', 'TradeFlex Trading']}
/>

<header class="store-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Courses', href: '/courses' }
			]}
		/>
		<div class="hero-grid" {@attach fadeUp({ y: 20 })}>
			<div>
				<Badge variant="gold">
					<IconBook2 size={12} />Course catalog
				</Badge>
				<h1>Education built for traders who learn by doing.</h1>
				<p>
					Lorem ipsum dolor sit amet, consectetur adipiscing elit. Every course is structured into
					modules, paired with interactive playgrounds, and capped with a real certification exam.
					No fluff. No filler. Just clarity.
				</p>
				<div class="hero-cta">
					<Button variant="primary" size="lg" href="#catalog">See the catalog</Button>
					<Button variant="gold-outline" size="lg" href="/free-guide"
						>Start with the free guide</Button
					>
				</div>
			</div>
			<aside class="hero-stats">
				<div class="hs"><strong>12</strong><span>modules per course</span></div>
				<div class="hs"><strong>Lifetime</strong><span>access &amp; updates</span></div>
				<div class="hs"><strong>Cohort</strong><span>monthly kickoffs</span></div>
				<div class="hs"><strong>Cert</strong><span>signed certificate</span></div>
			</aside>
		</div>
	</div>
</header>

<section class="section" id="catalog">
	<div class="container">
		<SectionHeading
			eyebrow="Available now"
			title="Sharpen one skill, deeply."
			subtitle="More courses on the way — but the foundation matters most."
		/>

		<div class="grid" {@attach stagger({ stagger: 0.1, y: 24 })}>
			{#each COURSES as p (p.id)}
				<ProductCard product={p} featured />
			{/each}
			{#each ROADMAP as r (r.title)}
				<article class="card roadmap-card" aria-label="Upcoming course: {r.title}">
					<div class="cover" style:background={r.posterColor}>
						<div class="cover-pattern" aria-hidden="true">
							<div class="rune"></div>
							<div class="rune r2"></div>
							<div class="rune r3"></div>
						</div>
						<div class="cover-kind">
							<span class="kind-icon" aria-hidden="true">
								<IconBook2 size={14} />
							</span>
							course
						</div>
						<div class="cover-badge">
							<Badge variant="gold">{r.quarter}</Badge>
						</div>
						<div class="cover-mark" aria-hidden="true">
							<IconRocket size={22} />
						</div>
					</div>

					<div class="body">
						<p class="eyebrow">Roadmap</p>
						<h3 class="name">{r.title}</h3>
						<p class="tagline">{r.tagline}</p>

						<div class="actions">
							<Button variant="gold-outline" size="md" href="/free-guide" fullWidth>
								Notify me
								{#snippet iconRight()}<IconArrowRight size={14} />{/snippet}
							</Button>
						</div>
					</div>
				</article>
			{/each}
		</div>
	</div>
</section>

<style>
	.store-hero {
		padding: clamp(4rem, 8vw, 6rem) 0 0;
		background: radial-gradient(ellipse at 30% 0%, rgba(232, 182, 96, 0.1), transparent 60%);
	}
	.hero-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		align-items: end;
		padding-top: var(--space-6);
	}
	@media (--bp-lg) {
		.hero-grid {
			grid-template-columns: 1.2fr 1fr;
		}
	}
	.hero-grid h1 {
		font-family: var(--font-display);
		font-size: clamp(2.5rem, 5vw, 4.5rem);
		margin: var(--space-5) 0 var(--space-4);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
	}
	.hero-grid p {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		max-width: 56ch;
	}
	.hero-cta {
		margin-top: var(--space-6);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
	}
	.hero-stats {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-4);
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
	}
	.hs {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.hs strong {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		line-height: 1;
	}
	.hs span {
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
	}

	.section {
		padding-block: clamp(4rem, 8vw, 8rem);
	}
	.grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-6);
		margin-top: clamp(2.5rem, 5vw, 4rem);
	}
	@media (--bp-md) {
		.grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (--bp-xl) {
		.grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	/* Roadmap card: visually mirrors ProductCard's shell. */
	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		overflow: hidden;
		transition:
			transform var(--dur-base) var(--ease-out),
			border-color var(--dur-base) var(--ease-out),
			box-shadow var(--dur-base) var(--ease-out);
	}
	.card:hover {
		border-color: var(--border-gold);
		transform: translateY(-4px);
		box-shadow: var(--shadow-elev-3), var(--glow-gold);
	}

	.cover {
		position: relative;
		aspect-ratio: 5 / 3;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		padding: var(--space-4);
		overflow: hidden;
	}
	.cover-pattern {
		position: absolute;
		inset: 0;
	}
	.rune {
		position: absolute;
		width: 240px;
		height: 240px;
		border: 1px solid rgba(232, 182, 96, 0.18);
		border-radius: 50%;
		top: -60%;
		right: -30%;
	}
	.rune.r2 {
		top: -40%;
		right: -10%;
		width: 180px;
		height: 180px;
		border-color: rgba(232, 182, 96, 0.1);
	}
	.rune.r3 {
		top: -20%;
		right: 10%;
		width: 120px;
		height: 120px;
		border-color: rgba(232, 182, 96, 0.05);
	}
	.cover-kind {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		padding: 4px 10px;
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(6px);
		color: var(--ink-100);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		font-weight: var(--weight-semibold);
		border-radius: var(--radius-full);
		width: max-content;
		border: 1px solid rgba(255, 255, 255, 0.08);
		position: relative;
		z-index: 1;
	}
	.kind-icon {
		display: inline-flex;
		align-items: center;
		color: var(--gold-300);
	}
	.cover-badge {
		position: absolute;
		top: var(--space-4);
		left: var(--space-4);
	}
	.cover-mark {
		position: absolute;
		top: var(--space-4);
		right: var(--space-4);
		width: 36px;
		height: 36px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(6px);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: var(--radius-full);
		color: var(--gold-300);
	}

	.body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		flex: 1;
	}
	.eyebrow {
		font-size: var(--text-2xs);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		margin: 0;
	}
	.eyebrow::before {
		display: none;
	}
	.name {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
		line-height: var(--leading-snug);
	}
	.tagline {
		color: var(--ink-300);
		font-size: var(--text-sm);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
	.actions {
		margin-top: auto;
	}
</style>
