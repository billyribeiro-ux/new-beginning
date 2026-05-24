<script lang="ts">
	import { IconArrowRight, IconSparkles } from '@tabler/icons-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { fadeUp, cursorGlow } from '$lib/animations/attachments.js';

	type Props = {
		eyebrow?: string;
		title: string;
		subtitle?: string;
		ctaLabel: string;
		ctaHref: string;
		secondaryLabel?: string;
		secondaryHref?: string;
	};
	let { eyebrow, title, subtitle, ctaLabel, ctaHref, secondaryLabel, secondaryHref }: Props =
		$props();
</script>

<section class="cta-banner" {@attach fadeUp({ y: 24 })} {@attach cursorGlow()}>
	<div class="halo"></div>
	<div class="inner">
		{#if eyebrow}
			<p class="eyebrow">
				<IconSparkles size={14} />
				{eyebrow}
			</p>
		{/if}
		<h2 class="title text-balance">{title}</h2>
		{#if subtitle}<p class="sub text-pretty">{subtitle}</p>{/if}
		<div class="actions">
			<Button variant="primary" size="xl" href={ctaHref}>
				{ctaLabel}
				{#snippet iconRight()}<IconArrowRight size={18} />{/snippet}
			</Button>
			{#if secondaryLabel && secondaryHref}
				<Button variant="gold-outline" size="xl" href={secondaryHref}>{secondaryLabel}</Button>
			{/if}
		</div>
	</div>
</section>

<style>
	.cta-banner {
		--glow-x: 50%;
		--glow-y: 50%;
		position: relative;
		isolation: isolate;
		padding: clamp(3rem, 6vw, 6rem) clamp(2rem, 5vw, 5rem);
		background: linear-gradient(160deg, var(--surface-2), var(--surface-1));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		overflow: hidden;
		text-align: center;
	}
	.cta-banner::before {
		content: '';
		position: absolute;
		inset: 0;
		background: radial-gradient(
			600px circle at var(--glow-x) var(--glow-y),
			rgba(232, 182, 96, 0.16),
			transparent 60%
		);
		z-index: -1;
		transition: background var(--dur-base) var(--ease-out);
	}
	.halo {
		position: absolute;
		inset: -40% -10% auto -10%;
		height: 80%;
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.18), transparent 60%);
		z-index: -1;
		pointer-events: none;
	}
	.inner {
		max-width: 760px;
		margin-inline: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
		align-items: center;
	}
	.eyebrow {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-2xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-300);
		padding: 6px 14px;
		background: linear-gradient(135deg, rgba(245, 208, 138, 0.14), rgba(176, 131, 47, 0.06));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-full);
		margin: 0;
	}
	.title {
		font-family: var(--font-display);
		font-size: clamp(2rem, 4vw, 3.5rem);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
		margin: 0;
	}
	.sub {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
		max-width: 56ch;
	}
	.actions {
		margin-top: var(--space-3);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
		justify-content: center;
	}
</style>
