<script lang="ts">
	import { IconEdit, IconStar, IconStarFilled, IconUsers } from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { DAY_TRADING_PLANS } from '$lib/data/plans.js';
	import { formatPrice } from '$lib/utils/money.js';

	const enrichedPlans = DAY_TRADING_PLANS.map((p) => ({
		...p,
		active: p.cadence === 'monthly' ? 412 : p.cadence === 'quarterly' ? 828 : 287,
		mrr: p.cadence === 'monthly' ? 101_764 : p.cadence === 'quarterly' ? 192_517 : 47_894
	}));
</script>

<Seo title="Admin · Subscription plans" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Catalog</p>
		<h2>Subscription plans</h2>
		<p class="muted">Day Trading service · {enrichedPlans.length} active tiers.</p>
	</div>
	<Button variant="primary">Create plan</Button>
</header>

<div class="grid">
	{#each enrichedPlans as p (p.id)}
		<article class="plan-card" class:is-featured={p.featured}>
			<header class="ph-card">
				<div class="row-between">
					<h3>{p.name}</h3>
					{#if p.featured}
						<Badge variant="gold" size="sm"><IconStarFilled size={10} />Featured</Badge>
					{:else}
						<Badge variant="outline" size="sm">{p.cadence}</Badge>
					{/if}
				</div>
				<p class="muted">{p.tagline}</p>
			</header>

			<div class="price">
				<span class="amt">{formatPrice(p.priceCents)}</span>
				<span class="cad">/ {p.cadence}</span>
			</div>

			<dl class="stats">
				<div>
					<dt><IconUsers size={12} />Active subscribers</dt>
					<dd>{p.active.toLocaleString()}</dd>
				</div>
				<div>
					<dt>Monthly equiv.</dt>
					<dd>${(p.monthlyEquivalentCents / 100).toFixed(0)}</dd>
				</div>
				<div>
					<dt>MRR contribution</dt>
					<dd>${(p.mrr / 100).toLocaleString()}</dd>
				</div>
				{#if p.savingsPct}<div>
						<dt>Savings vs monthly</dt>
						<dd>{p.savingsPct}%</dd>
					</div>{/if}
			</dl>

			<div class="actions">
				<Button variant="gold-outline" size="sm">
					{#snippet iconLeft()}<IconEdit size={14} />{/snippet}
					Edit plan
				</Button>
				<Button variant="ghost" size="sm">View members</Button>
			</div>
		</article>
	{/each}
</div>

<style>
	.ph {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: var(--space-4);
		flex-wrap: wrap;
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	.eyebrow {
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
	.ph h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: var(--space-2) 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}

	.grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
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

	.plan-card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.plan-card.is-featured {
		border-color: var(--border-gold);
		background: linear-gradient(160deg, rgba(232, 182, 96, 0.06), var(--surface-1));
	}
	.row-between {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-2);
	}
	.ph-card h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.ph-card .muted {
		margin-top: var(--space-2);
		font-size: var(--text-sm);
	}

	.price {
		display: flex;
		align-items: baseline;
		gap: var(--space-2);
	}
	.amt {
		font-family: var(--font-display);
		font-size: var(--text-4xl);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		font-weight: var(--weight-semibold);
	}
	.cad {
		font-size: var(--text-sm);
		color: var(--ink-400);
	}

	.stats {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.stats > div {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: var(--text-xs);
	}
	.stats dt {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		font-weight: var(--weight-semibold);
	}
	.stats dt :global(svg) {
		color: var(--gold-400);
	}
	.stats dd {
		margin: 0;
		font-family: var(--font-display);
		font-size: var(--text-md);
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
	}

	.actions {
		display: flex;
		gap: var(--space-2);
		margin-top: auto;
	}
</style>
