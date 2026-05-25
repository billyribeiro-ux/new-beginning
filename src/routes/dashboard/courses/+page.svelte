<script lang="ts">
	import {
		IconArrowRight,
		IconCertificate,
		IconClock,
		IconPlayerPlay,
		IconBookmark
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import ProgressBar from '$lib/components/dashboard/ProgressBar.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { COURSES } from '$lib/data/products.js';

	const enrollments = [
		{
			slug: 'options-101',
			name: 'Options 101',
			progress: 38,
			lastLesson: 'Module 3 · Delta',
			cohort: 'May 2026 cohort'
		}
	];
	const wishlist = COURSES.filter((c) => !enrollments.some((e) => e.slug === c.slug));
</script>

<Seo title="My Courses" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">Your library</p>
		<h2>Continue where you left off.</h2>
	</div>
</header>

<section class="cards-grid">
	{#each enrollments as e (e.slug)}
		{@const course = COURSES.find((c) => c.slug === e.slug)}
		{#if course}
			<article class="course-card">
				<div class="cover" style:background={course.media.posterColor}>
					<span class="cohort">{e.cohort}</span>
					<div class="play"><IconPlayerPlay size={28} /></div>
				</div>
				<div class="body">
					<h3>{course.name}</h3>
					<p class="muted">{course.tagline}</p>
					<div class="meta-row">
						<span><IconClock size={13} />~12 hours</span>
						<span><IconCertificate size={13} />Certificate</span>
					</div>
					<ProgressBar value={e.progress} label="Progress" />
					<p class="last">Up next: <strong>{e.lastLesson}</strong></p>
					<Button variant="primary" size="md" fullWidth href="/dashboard/courses/{course.slug}">
						Resume course
						{#snippet iconRight()}<IconArrowRight size={14} />{/snippet}
					</Button>
				</div>
			</article>
		{/if}
	{/each}
</section>

{#if wishlist.length > 0}
	<section class="suggest">
		<header><h3>Explore other courses</h3></header>
		<div class="suggest-grid">
			{#each wishlist as c (c.id)}
				<article class="sg-card">
					<div class="sg-thumb" style:background={c.media.posterColor}>
						<IconBookmark size={20} />
					</div>
					<div class="sg-body">
						<h4>{c.name}</h4>
						<p class="muted">{c.tagline}</p>
						<Button variant="gold-outline" size="sm" href="/courses/{c.slug}">View course</Button>
					</div>
				</article>
			{/each}
		</div>
	</section>
{/if}

<style>
	.ph {
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
		margin: var(--space-2) 0 0;
	}

	.cards-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	@media (--bp-md) {
		.cards-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (--bp-xl) {
		.cards-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.course-card {
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
	.course-card:hover {
		border-color: var(--border-gold);
	}
	.cover {
		position: relative;
		aspect-ratio: 16 / 9;
		display: flex;
		align-items: flex-start;
		justify-content: flex-start;
		padding: var(--space-4);
	}
	.cohort {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(8px);
		color: var(--ink-100);
		padding: 4px 10px;
		border-radius: var(--radius-full);
		border: 1px solid rgba(255, 255, 255, 0.08);
		font-weight: var(--weight-semibold);
	}
	.play {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--gold-300);
	}
	.body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		flex: 1;
	}
	.body h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.muted {
		color: var(--ink-300);
		font-size: var(--text-sm);
		margin: 0;
		line-height: var(--leading-relaxed);
	}
	.meta-row {
		display: flex;
		gap: var(--space-4);
	}
	.meta-row span {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
	.meta-row :global(svg) {
		color: var(--gold-400);
	}
	.last {
		font-size: var(--text-xs);
		color: var(--ink-300);
		margin: 0;
	}
	.last strong {
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
	}

	.suggest {
		padding-top: clamp(2rem, 4vw, 3rem);
		border-top: 1px solid var(--border-default);
	}
	.suggest header {
		margin-bottom: var(--space-5);
	}
	.suggest h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}

	.suggest-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
	}
	@media (--bp-md) {
		.suggest-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	.sg-card {
		display: grid;
		grid-template-columns: 80px 1fr;
		gap: var(--space-4);
		padding: var(--space-4);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.sg-thumb {
		width: 80px;
		height: 80px;
		border-radius: var(--radius-md);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--gold-300);
	}
	.sg-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		align-items: flex-start;
	}
	.sg-body h4 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 0;
	}
</style>
