<script lang="ts">
	import IconArrowRight from '@tabler/icons-svelte/icons/arrow-right';
	import IconRocket from '@tabler/icons-svelte/icons/rocket';
	import type { RoadmapItem } from '$lib/data/products.js';
	import Button from '$lib/components/ui/Button.svelte';

	type Props = { item: RoadmapItem };
	let { item }: Props = $props();
</script>

<article class="card">
	<div class="cover" style:background={item.posterColor}>
		<div class="cover-pattern" aria-hidden="true">
			<div class="rune"></div>
			<div class="rune r2"></div>
			<div class="rune r3"></div>
		</div>
		<div class="cs-mark" aria-hidden="true">
			<IconRocket size={24} />
		</div>
		<div class="cover-tag">Roadmap · {item.quarter}</div>
	</div>

	<div class="body">
		<h3 class="name">{item.name}</h3>
		<p class="tagline">{item.tagline}</p>
		<p class="description">{item.description}</p>

		<div class="actions">
			<Button variant="ghost" size="md" href="/free-guide" fullWidth>
				Be notified
				{#snippet iconRight()}<IconArrowRight size={14} />{/snippet}
			</Button>
		</div>
	</div>
</article>

<style>
	.card {
		container-type: inline-size;
		container-name: roadmap-card;

		position: relative;
		display: flex;
		flex-direction: column;
		background: var(--surface-1);
		border: 1px dashed var(--border-default);
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
		box-shadow: var(--shadow-elev-3);
	}

	.cover {
		position: relative;
		aspect-ratio: 5 / 3;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
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
		border: 1px solid rgba(232, 182, 96, 0.12);
		border-radius: 50%;
		top: -60%;
		right: -30%;
	}
	.rune.r2 {
		top: -40%;
		right: -10%;
		width: 180px;
		height: 180px;
		border-color: rgba(232, 182, 96, 0.07);
	}
	.rune.r3 {
		top: -20%;
		right: 10%;
		width: 120px;
		height: 120px;
		border-color: rgba(232, 182, 96, 0.04);
	}
	.cs-mark {
		position: relative;
		width: 48px;
		height: 48px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		color: var(--gold-400);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-md);
	}
	.cover-tag {
		position: relative;
		align-self: flex-start;
		padding: 4px 10px;
		background: rgba(0, 0, 0, 0.85);
		color: var(--gold-300);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		font-weight: var(--weight-semibold);
		border-radius: var(--radius-full);
		border: 1px solid rgba(255, 255, 255, 0.08);
		width: max-content;
	}
	@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)) {
		.cover-tag {
			background: rgba(0, 0, 0, 0.55);
			backdrop-filter: blur(6px);
			-webkit-backdrop-filter: blur(6px);
		}
	}

	.body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		flex: 1;
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
	.description {
		color: var(--ink-400);
		font-size: var(--text-xs);
		line-height: var(--leading-relaxed);
		margin: 0;
		padding-block: var(--space-4);
		border-block: 1px dashed var(--border-default);
	}
	.actions {
		margin-top: auto;
	}
</style>
