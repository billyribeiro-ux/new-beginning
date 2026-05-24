<script lang="ts">
	import {
		IconArrowRight,
		IconStarFilled,
		IconBookmark,
		IconChartCandle,
		IconBook2
	} from '@tabler/icons-svelte';
	import type { Product } from '$lib/data/products.js';
	import PriceTag from './PriceTag.svelte';
	import AddToCartButton from './AddToCartButton.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { discountPct } from '$lib/utils/money.js';
	import { resolve } from '$app/paths';

	type Props = { product: Product; featured?: boolean };
	let { product, featured = false }: Props = $props();

	const detailHref = $derived(
		product.kind === 'indicator'
			? resolve('/indicators/[slug]', { slug: product.slug })
			: resolve('/courses/[slug]', { slug: product.slug })
	);
	const discount = $derived(
		product.originalPriceCents ? discountPct(product.originalPriceCents, product.priceCents) : 0
	);
</script>

<article class="card" class:is-featured={featured}>
	<div class="cover" style:background={product.media.posterColor}>
		<div class="cover-pattern" aria-hidden="true">
			<div class="rune"></div>
			<div class="rune r2"></div>
			<div class="rune r3"></div>
		</div>
		<div class="cover-kind">
			<span class="kind-icon" aria-hidden="true">
				{#if product.kind === 'indicator'}
					<IconChartCandle size={14} />
				{:else}
					<IconBook2 size={14} />
				{/if}
			</span>
			{product.kind}
		</div>
		{#if product.badge}
			<div class="cover-badge">
				<Badge variant="gold">{product.badge}</Badge>
			</div>
		{/if}
		<button class="bookmark" type="button" aria-label="Save for later">
			<IconBookmark size={16} />
		</button>
	</div>

	<div class="body">
		<div class="meta">
			<div class="rating" aria-label="Rated {product.rating.value} out of 5">
				<IconStarFilled size={14} />
				<span>{product.rating.value.toFixed(1)}</span>
				<span class="count">· {product.rating.count.toLocaleString()} reviews</span>
			</div>
			{#if discount > 0}<Badge variant="success">Save {discount}%</Badge>{/if}
		</div>

		<h3 class="name">
			<a href={detailHref}>{product.name}</a>
		</h3>
		<p class="tagline">{product.tagline}</p>

		<ul class="highlights">
			{#each product.highlights.slice(0, 3) as h (h)}
				<li>{h}</li>
			{/each}
		</ul>

		<div class="price-row">
			<PriceTag cents={product.priceCents} originalCents={product.originalPriceCents} size="md" />
			<a class="detail-link" href={detailHref} aria-label="View {product.name}">
				Details
				<IconArrowRight size={14} />
			</a>
		</div>

		<div class="actions">
			<AddToCartButton
				line={{
					id: `prod-${product.id}`,
					kind: 'product',
					slug: product.slug,
					name: product.name,
					subtitle: product.kind === 'indicator' ? 'Indicator · Lifetime' : 'Course · Lifetime',
					priceCents: product.priceCents
				}}
				variant="primary"
				size="md"
				fullWidth
			/>
		</div>
	</div>
</article>

<style>
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
	.is-featured {
		border-color: var(--border-gold);
		box-shadow: var(--glow-gold);
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
	.bookmark {
		position: absolute;
		top: var(--space-4);
		right: var(--space-4);
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(6px);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: var(--radius-full);
		color: var(--ink-200);
	}
	.bookmark:hover {
		color: var(--gold-300);
		background: rgba(0, 0, 0, 0.7);
	}

	.body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		flex: 1;
	}
	.meta {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.rating {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--gold-400);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
	}
	.rating .count {
		color: var(--ink-400);
		font-weight: var(--weight-regular);
		margin-left: 4px;
	}

	.name {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
		line-height: var(--leading-snug);
	}
	.name a {
		color: inherit;
		text-decoration: none;
		transition: color var(--dur-fast) var(--ease-out);
	}
	.name a:hover {
		color: var(--gold-300);
	}

	.tagline {
		color: var(--ink-300);
		font-size: var(--text-sm);
		line-height: var(--leading-relaxed);
		margin: 0;
	}

	.highlights {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		padding: var(--space-4) 0;
		border-block: 1px dashed var(--border-default);
	}
	.highlights li {
		position: relative;
		padding-left: var(--space-5);
		font-size: var(--text-xs);
		color: var(--ink-300);
	}
	.highlights li::before {
		content: '';
		position: absolute;
		left: 4px;
		top: 6px;
		width: 6px;
		height: 6px;
		background: var(--gold-500);
		border-radius: 50%;
	}

	.price-row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}
	.detail-link {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		color: var(--ink-300);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		text-decoration: none;
		transition: all var(--dur-fast) var(--ease-out);
	}
	.detail-link:hover {
		color: var(--gold-300);
		gap: var(--space-2);
	}

	.actions {
		margin-top: auto;
	}
</style>
