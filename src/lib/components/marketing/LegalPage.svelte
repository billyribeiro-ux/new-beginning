<script lang="ts">
	import type { Snippet } from 'svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';

	type Props = { title: string; updated: string; href: string; children: Snippet };
	let { title, updated, href, children }: Props = $props();
</script>

<section class="legal">
	<div class="container-narrow">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: title, href }
			]}
		/>
		<header>
			<p class="updated">Last updated · {updated}</p>
			<h1>{title}</h1>
		</header>
		<article class="prose">
			{@render children()}
		</article>
	</div>
</section>

<style>
	.legal {
		padding-block: clamp(3rem, 6vw, 5rem);
	}
	header {
		margin: var(--space-6) 0 clamp(2rem, 5vw, 3.5rem);
	}
	.updated {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0 0 var(--space-3);
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2.25rem, 4vw, 3.5rem);
		margin: 0;
		letter-spacing: var(--tracking-tight);
	}
	.prose {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.prose :global(h2) {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin-top: var(--space-8);
		color: var(--ink-100);
	}
	.prose :global(h3) {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin-top: var(--space-6);
		color: var(--ink-100);
	}
	.prose :global(p) {
		color: var(--ink-200);
		font-size: var(--text-md);
		line-height: var(--leading-relaxed);
	}
	.prose :global(ul),
	.prose :global(ol) {
		padding-inline-start: var(--space-6);
		color: var(--ink-200);
		font-size: var(--text-md);
		line-height: var(--leading-relaxed);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.prose :global(a) {
		color: var(--gold-300);
		text-decoration: underline;
	}
	.prose :global(strong) {
		color: var(--ink-100);
	}
</style>
