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
				<Badge variant="gold"
					>{#snippet children()}<IconBook2 size={12} />Course catalog{/snippet}</Badge
				>
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
				<div class="hs"><strong>Lifetime</strong><span>access & updates</span></div>
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
			{#each COURSES as p}
				<ProductCard product={p} featured />
			{/each}
			<article class="coming-soon">
				<div class="cs-mark"><IconRocket size={28} /></div>
				<p class="eyebrow">Roadmap · Q3 2026</p>
				<h3>Futures 201: Market Structure</h3>
				<p>
					Order flow, market profile, and intraday structure for ES/NQ/CL specialists. Deep, dense,
					and cohort-led.
				</p>
				<Button variant="ghost" size="sm" href="/free-guide">
					Notify me
					{#snippet iconRight()}<IconArrowRight size={14} />{/snippet}
				</Button>
			</article>
			<article class="coming-soon">
				<div class="cs-mark"><IconRocket size={28} /></div>
				<p class="eyebrow">Roadmap · Q4 2026</p>
				<h3>Volatility Trader Bootcamp</h3>
				<p>
					From IV term structure to gamma scalping. A focused, intensive course for the systematic
					vol seller.
				</p>
				<Button variant="ghost" size="sm" href="/free-guide">
					Notify me
					{#snippet iconRight()}<IconArrowRight size={14} />{/snippet}
				</Button>
			</article>
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
	@media (min-width: 1024px) {
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
	@media (min-width: 768px) {
		.grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (min-width: 1280px) {
		.grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.coming-soon {
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px dashed var(--border-default);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.cs-mark {
		width: 56px;
		height: 56px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		color: var(--gold-400);
		border-radius: var(--radius-lg);
		margin-bottom: var(--space-3);
	}
	.coming-soon h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.coming-soon p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
	.coming-soon .eyebrow {
		font-size: var(--text-2xs);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		margin: 0;
	}
	.coming-soon .eyebrow::before {
		display: none;
	}
</style>
