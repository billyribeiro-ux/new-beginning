<script lang="ts">
	import { IconSparkles, IconArrowNarrowRight } from '@tabler/icons-svelte';
	import { resolve } from '$app/paths';
</script>

<div class="announcement" role="region" aria-label="Site announcement">
	<div class="inner">
		<span class="icon"><IconSparkles size={14} /></span>
		<p>
			<strong>New cohort just opened</strong> · Options 101 spring intake — limited seats
		</p>
		<a class="cta" href={resolve('/courses/[slug]', { slug: 'options-101' })}>
			Reserve a spot
			<IconArrowNarrowRight size={14} />
		</a>
	</div>
</div>

<style>
	.announcement {
		background: linear-gradient(
			90deg,
			rgba(232, 182, 96, 0.06),
			rgba(176, 131, 47, 0.04),
			rgba(232, 182, 96, 0.06)
		);
		border-bottom: 1px solid var(--border-subtle);
		font-size: var(--text-xs);
		color: var(--ink-200);
	}
	.inner {
		max-width: var(--container-max);
		margin-inline: auto;
		padding: var(--space-2) var(--container-gutter);
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		min-height: var(--announcement-height);
		flex-wrap: wrap;
		text-align: center;
	}
	p {
		margin: 0;
		color: var(--ink-300);
		font-size: var(--text-xs);
		line-height: 1.4;
	}
	strong {
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
	}
	.icon {
		color: var(--gold-400);
		display: inline-flex;
		align-items: center;
	}
	.cta {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--gold-300);
		font-weight: var(--weight-semibold);
		text-decoration: none;
		transition:
			gap var(--dur-base) var(--ease-out),
			color var(--dur-base) var(--ease-out);
	}
	.cta:hover {
		color: var(--gold-200);
		gap: 8px;
	}
	@media (max-width: 640px) {
		.cta {
			display: none;
		}
	}
</style>
