<script lang="ts">
	import Seo from '$lib/components/seo/Seo.svelte';
	import SectionHeading from '$lib/components/marketing/SectionHeading.svelte';
	import ProductCard from '$lib/components/commerce/ProductCard.svelte';
	import RoadmapCard from '$lib/components/commerce/RoadmapCard.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import { INDICATORS, INDICATOR_ROADMAP } from '$lib/data/products.js';
	import { stagger, fadeUp } from '$lib/animations/attachments.js';
	import { IconChartCandle } from '@tabler/icons-svelte';
	import Button from '$lib/components/ui/Button.svelte';
</script>

<Seo
	title="Indicators"
	description="Professional-grade trading indicators built for NinjaTrader, TradingView and ThinkOrSwim. No repaint. Lifetime updates."
	keywords={['trading indicators', 'NinjaTrader', 'TradingView', 'Revolution Ranger']}
/>

<header class="store-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Indicators', href: '/indicators' }
			]}
		/>
		<div class="hero-grid" {@attach fadeUp({ y: 20 })}>
			<div>
				<Badge variant="gold">
					<IconChartCandle size={12} />Indicator store
				</Badge>
				<h1>Indicators built for traders who measure the tape.</h1>
				<p>
					Lorem ipsum dolor sit amet, consectetur adipiscing elit. Every TradeFlex indicator is
					engineered live on the desk, validated across regimes, and shipped with lifetime updates.
					No repaint. No smoke. Just edge.
				</p>
				<div class="hero-cta">
					<Button variant="primary" size="lg" href="#catalog">Browse the catalog</Button>
					<Button variant="gold-outline" size="lg" href="/free-guide">Free Greeks Guide</Button>
				</div>
			</div>
			<aside class="hero-stats">
				<div class="hs"><strong>3-min</strong><span>average install</span></div>
				<div class="hs"><strong>Lifetime</strong><span>free updates</span></div>
				<div class="hs"><strong>14-day</strong><span>refund window</span></div>
				<div class="hs"><strong>40+</strong><span>instruments supported</span></div>
			</aside>
		</div>
	</div>
</header>

<section class="section" id="catalog">
	<div class="container">
		<SectionHeading
			eyebrow="The catalog"
			title="Engineered tools, ready to deploy."
			subtitle="Hand-built indicators shipped for the platforms you already use. Bundle with the Day Trading subscription for the full stack."
		/>

		<div class="grid" {@attach stagger({ stagger: 0.1, y: 24 })}>
			{#each INDICATORS as p (p.id)}
				<ProductCard product={p} featured />
			{/each}
			{#each INDICATOR_ROADMAP as item (item.id)}
				<RoadmapCard {item} />
			{/each}
		</div>
	</div>
</section>

<style>
	.store-hero {
		padding: clamp(4rem, 8vw, 6rem) 0 0;
		background: radial-gradient(ellipse at 70% 0%, rgba(232, 182, 96, 0.1), transparent 60%);
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
	@media (--bp-3xl) {
		.grid {
			grid-template-columns: repeat(4, 1fr);
		}
	}

</style>
