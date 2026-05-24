<script lang="ts">
	import type { Snippet } from 'svelte';
	import { fadeUp } from '$lib/animations/attachments.js';

	type Props = {
		eyebrow?: string;
		title: string;
		subtitle?: string;
		align?: 'left' | 'center';
		size?: 'md' | 'lg';
		actions?: Snippet;
	};
	let { eyebrow, title, subtitle, align = 'left', size = 'lg', actions }: Props = $props();
</script>

<header class="heading align-{align} size-{size}" {@attach fadeUp({ y: 18 })}>
	{#if eyebrow}<p class="eyebrow">{eyebrow}</p>{/if}
	<h2 class="title text-balance">{title}</h2>
	{#if subtitle}<p class="subtitle text-pretty">{subtitle}</p>{/if}
	{#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
	.heading {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		max-width: 64ch;
	}
	.align-center {
		text-align: center;
		margin-inline: auto;
		align-items: center;
	}
	.title {
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
		color: var(--ink-100);
		margin: 0;
	}
	.size-md .title {
		font-size: var(--text-3xl);
	}
	.size-lg .title {
		font-size: var(--text-4xl);
	}
	.subtitle {
		color: var(--ink-300);
		font-size: var(--text-md);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
	.actions {
		margin-top: var(--space-3);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
	}
	.align-center .actions {
		justify-content: center;
	}
</style>
