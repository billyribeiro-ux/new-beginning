<script lang="ts">
	import type { PageData } from './$types';
	import { resolve } from '$app/paths';
	import {
		IconCircleCheckFilled,
		IconDownload,
		IconBookmark,
		IconArrowRight,
		IconMail
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { fadeUp } from '$lib/animations/attachments.js';

	let { data }: { data: PageData } = $props();
</script>

<Seo title="You are in" description="Your free Options Greeks Guide is ready." noindex />

<section class="success">
	<div class="container">
		<div class="card" {@attach fadeUp({ y: 24 })}>
			<div class="success-icon">
				<IconCircleCheckFilled size={56} />
			</div>
			<p class="eyebrow">Delivered · Free guide</p>
			<h1>You are in.</h1>
			<p class="lead">
				The Options Greeks Guide is on its way to <strong>{data.email || 'your inbox'}</strong>.
				While you wait, grab the PDF directly below — no email round-trip required.
			</p>

			<div class="actions">
				<Button variant="primary" size="lg" href="/pdf/options-greeks-guide.pdf">
					{#snippet iconLeft()}<IconDownload size={16} />{/snippet}
					Download the PDF (1.2 MB)
				</Button>
				<Button variant="gold-outline" size="lg" href="/subscription">
					Explore the Day Trading desk
					{#snippet iconRight()}<IconArrowRight size={16} />{/snippet}
				</Button>
			</div>

			<div class="next">
				<h2>While you are here…</h2>
				<div class="next-grid">
					<a class="next-card" href={resolve('/subscription')}>
						<div class="ic"><IconMail size={20} /></div>
						<h3>Get the daily briefing</h3>
						<p>
							Pre-market notes for serious operators — included with the Day Trading subscription.
						</p>
					</a>
					<a class="next-card" href={resolve('/indicators/[slug]', { slug: 'revolution-ranger' })}>
						<div class="ic"><IconBookmark size={20} /></div>
						<h3>Revolution Ranger</h3>
						<p>The compression-to-expansion engine our desk runs every session.</p>
					</a>
					<a class="next-card" href={resolve('/courses/[slug]', { slug: 'options-101' })}>
						<div class="ic"><IconBookmark size={20} /></div>
						<h3>Options 101</h3>
						<p>Twelve modules from greeks to verticals to butterflies — exam included.</p>
					</a>
				</div>
			</div>
		</div>
	</div>
</section>

<style>
	.success {
		padding-block: clamp(4rem, 8vw, 7rem);
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.16), transparent 60%);
		min-height: calc(100dvh - var(--navbar-height));
	}
	.card {
		max-width: 760px;
		margin-inline: auto;
		text-align: center;
		padding: clamp(2.5rem, 5vw, 4rem);
		background: linear-gradient(160deg, var(--surface-1), var(--surface-2));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		box-shadow: var(--shadow-elev-3), var(--glow-gold);
	}
	.success-icon {
		display: inline-flex;
		width: 96px;
		height: 96px;
		align-items: center;
		justify-content: center;
		background: var(--gradient-gold);
		color: var(--surface-0);
		border-radius: var(--radius-full);
		margin-bottom: var(--space-5);
		box-shadow: 0 12px 32px rgba(212, 162, 76, 0.4);
	}
	.eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0 0 var(--space-3);
	}
	.eyebrow::before {
		display: none;
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2.25rem, 4vw, 3.5rem);
		margin: 0 0 var(--space-4);
		line-height: var(--leading-tight);
	}
	.lead {
		font-size: var(--text-md);
		color: var(--ink-300);
		max-width: 56ch;
		margin: 0 auto var(--space-6);
		line-height: var(--leading-relaxed);
	}
	.lead strong {
		color: var(--ink-100);
	}
	.actions {
		display: flex;
		gap: var(--space-3);
		justify-content: center;
		flex-wrap: wrap;
	}
	.next {
		margin-top: clamp(2.5rem, 5vw, 4rem);
		padding-top: clamp(2.5rem, 5vw, 4rem);
		border-top: 1px solid var(--border-default);
	}
	.next h2 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0 0 var(--space-5);
	}
	.next-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-3);
		text-align: left;
	}
	@media (--bp-md) {
		.next-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}
	.next-card {
		padding: var(--space-5);
		background: var(--surface-0);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		text-decoration: none;
		color: var(--ink-100);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		transition: all var(--dur-base) var(--ease-out);
	}
	.next-card:hover {
		border-color: var(--border-gold);
		transform: translateY(-2px);
	}
	.next-card .ic {
		width: 36px;
		height: 36px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		color: var(--gold-400);
		border-radius: var(--radius-md);
		margin-bottom: var(--space-2);
	}
	.next-card h3 {
		font-family: var(--font-display);
		font-size: var(--text-md);
		margin: 0;
	}
	.next-card p {
		font-size: var(--text-xs);
		color: var(--ink-300);
		margin: 0;
		line-height: var(--leading-relaxed);
	}
</style>
