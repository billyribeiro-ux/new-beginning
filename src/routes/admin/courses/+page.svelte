<script lang="ts">
	import {
		IconPlus,
		IconEdit,
		IconBook2,
		IconClock,
		IconUsers,
		IconStarFilled
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { COURSES } from '$lib/data/products.js';
	import { formatPrice } from '$lib/utils/money.js';

	const rows = COURSES.map((c) => ({
		...c,
		modules: 12,
		enrolled: c.slug === 'options-101' ? 612 : 0,
		duration: '~12 hours'
	}));
</script>

<Seo title="Admin · Courses" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Catalog</p>
		<h2>Courses</h2>
		<p class="muted">{rows.length} active courses · modules and lessons are managed inline.</p>
	</div>
	<Button variant="primary" size="md" href="/admin/products/new">
		{#snippet iconLeft()}<IconPlus size={14} />{/snippet}
		New course
	</Button>
</header>

<div class="grid">
	{#each rows as c (c.id)}
		<article class="course-card">
			<div class="cover" style:background={c.media.posterColor}>
				<span class="kc"><IconBook2 size={14} />Course</span>
				<Badge variant="success" size="sm">Published</Badge>
			</div>
			<div class="body">
				<header class="ch">
					<div>
						<h3>{c.name}</h3>
						<p class="muted">{c.slug}</p>
					</div>
					<strong class="price">{formatPrice(c.priceCents)}</strong>
				</header>
				<p class="desc">{c.tagline}</p>
				<dl class="meta">
					<div>
						<dt><IconClock size={12} />Length</dt>
						<dd>{c.duration}</dd>
					</div>
					<div>
						<dt><IconUsers size={12} />Enrolled</dt>
						<dd>{c.enrolled.toLocaleString()}</dd>
					</div>
					<div>
						<dt><IconBook2 size={12} />Modules</dt>
						<dd>{c.modules}</dd>
					</div>
					<div>
						<dt><IconStarFilled size={12} />Rating</dt>
						<dd>{c.rating.value.toFixed(1)} ({c.rating.count})</dd>
					</div>
				</dl>
				<div class="actions">
					<Button variant="gold-outline" size="sm" href="/admin/products/{c.id}">
						{#snippet iconLeft()}<IconEdit size={14} />{/snippet}
						Edit content
					</Button>
					<Button variant="ghost" size="sm" href="/courses/{c.slug}">Preview</Button>
				</div>
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
	@media (--bp-md) {
		.grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (--bp-2xl) {
		.grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.course-card {
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		overflow: hidden;
	}
	.course-card:hover {
		border-color: var(--border-gold);
	}
	.cover {
		aspect-ratio: 16 / 5;
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		padding: var(--space-4);
	}
	.kc {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(8px);
		color: var(--ink-100);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		font-weight: var(--weight-semibold);
		border-radius: var(--radius-full);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}
	.kc :global(svg) {
		color: var(--gold-300);
	}
	.body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.ch {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-3);
	}
	.ch h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.ch .muted {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		margin: 4px 0 0;
	}
	.price {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		color: var(--ink-100);
	}
	.desc {
		font-size: var(--text-sm);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
	.meta {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-3);
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.meta > div {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.meta dt {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.meta dt :global(svg) {
		color: var(--gold-400);
	}
	.meta dd {
		margin: 0;
		font-size: var(--text-sm);
		color: var(--ink-100);
		font-weight: var(--weight-medium);
	}
	.actions {
		display: flex;
		gap: var(--space-2);
	}
</style>
