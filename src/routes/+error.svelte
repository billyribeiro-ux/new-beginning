<script lang="ts">
	import { page } from '$app/state';
	import { IconHomeQuestion, IconArrowLeft } from '@tabler/icons-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import { resolve } from '$app/paths';
</script>

<Seo title={`${page.status} · Page not found`} noindex />

<section class="err-page">
	<div class="container">
		<div class="inner">
			<div class="icon-wrap"><IconHomeQuestion size={48} /></div>
			<p class="status">Error {page.status}</p>
			<h1>{page.error?.message ?? 'Something went off-script.'}</h1>
			<p class="copy">
				The page you were looking for is no longer here. Try one of the routes below, or head home
				and start fresh.
			</p>
			<div class="actions">
				<Button variant="primary" size="lg" href="/">
					{#snippet iconLeft()}<IconArrowLeft size={16} />{/snippet}
					Return home
				</Button>
				<Button variant="gold-outline" size="lg" href="/contact">Contact support</Button>
			</div>
			<div class="links">
				<a href={resolve('/subscription')}>Subscription</a>
				<a href={resolve('/indicators')}>Indicators</a>
				<a href={resolve('/courses')}>Courses</a>
				<a href={resolve('/free-guide')}>Free Greeks Guide</a>
			</div>
		</div>
	</div>
</section>

<style>
	.err-page {
		min-height: calc(100dvh - var(--navbar-height));
		display: flex;
		align-items: center;
		padding-block: clamp(4rem, 8vw, 6rem);
		background:
			radial-gradient(ellipse at top, rgba(232, 182, 96, 0.08), transparent 60%), var(--surface-0);
	}
	.inner {
		max-width: 640px;
		margin-inline: auto;
		text-align: center;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
	}
	.icon-wrap {
		width: 96px;
		height: 96px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.16), rgba(176, 131, 47, 0.04));
		color: var(--gold-300);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		margin-bottom: var(--space-2);
	}
	.status {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		margin: 0;
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2rem, 4vw, 3.5rem);
		margin: 0;
		line-height: var(--leading-tight);
	}
	.copy {
		color: var(--ink-300);
		max-width: 48ch;
		line-height: var(--leading-relaxed);
	}
	.actions {
		margin-top: var(--space-4);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
		justify-content: center;
	}
	.links {
		margin-top: var(--space-6);
		display: flex;
		gap: var(--space-5);
		flex-wrap: wrap;
		justify-content: center;
	}
	.links a {
		color: var(--ink-400);
		font-size: var(--text-sm);
		text-decoration: none;
		border-bottom: 1px dashed var(--border-default);
		padding-bottom: 2px;
	}
	.links a:hover {
		color: var(--gold-300);
		border-color: var(--gold-500);
	}
</style>
