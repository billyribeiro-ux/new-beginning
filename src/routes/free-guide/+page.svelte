<script lang="ts">
	import IconCheck from '@tabler/icons-svelte/icons/check';
	import IconDownload from '@tabler/icons-svelte/icons/download';
	import IconStarFilled from '@tabler/icons-svelte/icons/star-filled';
	import IconShieldCheck from '@tabler/icons-svelte/icons/shield-check';
	import IconBolt from '@tabler/icons-svelte/icons/bolt';
	import IconBookmark from '@tabler/icons-svelte/icons/bookmark';
	import Seo from '$lib/components/seo/Seo.svelte';
	import JsonLd from '$lib/components/seo/JsonLd.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import EmailCaptureForm from '$lib/components/forms/EmailCaptureForm.svelte';
	import { articleLd } from '$lib/utils/jsonld.js';
	import { fadeUp, stagger } from '$lib/animations/attachments.js';

	const chapters = [
		{ n: '01', t: 'What greeks measure (and what they do not)' },
		{ n: '02', t: 'Delta: directional exposure, decoded' },
		{ n: '03', t: 'Gamma: the second derivative you actually care about' },
		{ n: '04', t: 'Theta: the truth about time decay' },
		{ n: '05', t: 'Vega: the volatility tax (and benefit)' },
		{ n: '06', t: 'Worked examples on SPY, AAPL, NVDA' }
	];
</script>

<Seo
	title="Free Options Greeks Guide"
	description="A 24-page PDF that demystifies delta, gamma, theta, vega, and rho — with worked examples you can rebuild in your platform."
	type="article"
	keywords={['options greeks', 'free guide', 'delta gamma theta vega', 'options trading']}
/>

<JsonLd
	data={articleLd({
		headline: 'Options Greeks Guide',
		description: 'A 24-page foundational PDF on the options greeks.',
		url: '/free-guide',
		author: 'TradeFlex Trading'
	})}
/>

<section class="lm-hero">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Free Guide', href: '/free-guide' }
			]}
		/>

		<div class="hero-grid">
			<div class="left" {@attach fadeUp({ y: 20 })}>
				<Badge variant="gold">
					<IconStarFilled size={12} />Free · 24-page PDF
				</Badge>
				<h1>The Options Greeks Guide we wish we had on day one.</h1>
				<p class="lead">
					Lorem ipsum dolor sit amet, consectetur adipiscing elit. A visual-first treatment of the
					greeks — delta, gamma, theta, vega, rho — with worked examples on SPY, AAPL, and NVDA.
					Twenty-four pages. Plain language. Zero upsell.
				</p>
				<ul class="bullets" {@attach stagger({ stagger: 0.06, y: 12 })}>
					<li>
						<span class="ck"><IconCheck size={12} stroke={3} /></span>Sketch-style intuitions of
						each greek
					</li>
					<li>
						<span class="ck"><IconCheck size={12} stroke={3} /></span>Real payoff diagrams you can
						rebuild
					</li>
					<li>
						<span class="ck"><IconCheck size={12} stroke={3} /></span>Position sizing &
						assignment-risk cheat sheet
					</li>
					<li>
						<span class="ck"><IconCheck size={12} stroke={3} /></span>Glossary of every term you
						will ever read
					</li>
				</ul>
			</div>

			<aside class="form-card" {@attach fadeUp({ y: 28, delay: 0.15 })}>
				<div class="form-poster">
					<div class="poster-grid"></div>
					<div class="poster-cover">
						<p class="cover-eyebrow">Pocket Guide</p>
						<h3>The Greeks</h3>
						<p class="cover-sub">For traders who measure</p>
						<span class="cover-pages">24 pages</span>
					</div>
				</div>
				<div class="form-body">
					<h2>Get the PDF — instantly.</h2>
					<p class="muted">
						Enter your email. We send the download link to your inbox. No spam, no upsell.
					</p>
					<EmailCaptureForm
						action="?/capture"
						source="free-guide"
						ctaLabel="Send me the guide"
						successMessage="Check your inbox — the PDF is on its way."
					/>
					<ul class="trust">
						<li><IconShieldCheck size={14} /> Your email stays private. Unsubscribe anytime.</li>
						<li><IconBolt size={14} /> Instant delivery to your inbox.</li>
						<li><IconBookmark size={14} /> Lifetime access to free updates.</li>
					</ul>
				</div>
			</aside>
		</div>
	</div>
</section>

<section class="section">
	<div class="container">
		<div class="toc-grid" {@attach fadeUp({ y: 20 })}>
			<div>
				<p class="eyebrow">Inside the guide</p>
				<h2>Six chapters. One foundation.</h2>
				<p class="muted">
					Lorem ipsum dolor sit amet. A short, dense, visual-first read you can finish in a single
					evening — and reference for the rest of your career.
				</p>
			</div>
			<ol class="toc">
				{#each chapters as ch (ch.n)}
					<li>
						<span class="n">{ch.n}</span>
						<span class="t">{ch.t}</span>
					</li>
				{/each}
			</ol>
		</div>
	</div>
</section>

<style>
	.lm-hero {
		padding: clamp(3rem, 6vw, 5rem) 0;
		background: radial-gradient(ellipse at center top, rgba(232, 182, 96, 0.12), transparent 60%);
	}
	.hero-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		margin-top: var(--space-6);
		align-items: start;
	}
	@media (--bp-lg) {
		.hero-grid {
			grid-template-columns: 1.1fr 1fr;
		}
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2.25rem, 4vw, 3.75rem);
		margin: var(--space-5) 0 var(--space-4);
		letter-spacing: var(--tracking-tight);
		line-height: var(--leading-tight);
	}
	.lead {
		font-size: var(--text-md);
		color: var(--ink-300);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
	.bullets {
		list-style: none;
		margin-top: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.bullets li {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.ck {
		display: inline-flex;
		width: 22px;
		height: 22px;
		align-items: center;
		justify-content: center;
		background: rgba(232, 182, 96, 0.18);
		color: var(--gold-300);
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}

	.form-card {
		display: flex;
		flex-direction: column;
		background: var(--surface-1);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		overflow: hidden;
		box-shadow: var(--shadow-elev-3), var(--glow-gold);
	}
	.form-poster {
		aspect-ratio: 5 / 3;
		position: relative;
		background: linear-gradient(160deg, #2a1f0f, var(--surface-0));
		overflow: hidden;
		display: grid;
		place-items: center;
		padding: var(--space-6);
	}
	.poster-grid {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgba(232, 182, 96, 0.06) 1px, transparent 1px),
			linear-gradient(90deg, rgba(232, 182, 96, 0.06) 1px, transparent 1px);
		background-size: 24px 24px;
	}
	.poster-cover {
		position: relative;
		z-index: 1;
		text-align: center;
		padding: var(--space-8);
		background: linear-gradient(180deg, rgba(248, 246, 241, 0.04), rgba(248, 246, 241, 0.01));
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-elev-3);
	}
	.cover-eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		margin: 0 0 var(--space-2);
		font-weight: var(--weight-semibold);
	}
	.poster-cover h3 {
		font-family: var(--font-display);
		font-size: var(--text-4xl);
		margin: 0 0 var(--space-1);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
	}
	.cover-sub {
		font-size: var(--text-xs);
		color: var(--ink-300);
		margin: 0 0 var(--space-4);
		font-style: italic;
	}
	.cover-pages {
		display: inline-block;
		padding: 4px 10px;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-200);
		background: var(--surface-3);
		border-radius: var(--radius-full);
	}

	.form-body {
		padding: var(--space-7);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.form-body h2 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: 0;
	}
	.muted {
		color: var(--ink-300);
		font-size: var(--text-sm);
		margin: 0;
	}
	.trust {
		list-style: none;
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-default);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		margin-top: var(--space-2);
	}
	.trust li {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
	.trust li :global(svg) {
		color: var(--gold-400);
		flex-shrink: 0;
	}

	.section {
		padding-block: clamp(4rem, 8vw, 7rem);
	}
	.toc-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4rem);
		align-items: start;
	}
	@media (--bp-lg) {
		.toc-grid {
			grid-template-columns: 1fr 1.4fr;
		}
	}
	.eyebrow {
		font-size: var(--text-2xs);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		margin: 0;
	}
	.eyebrow::before {
		display: none;
	}
	.toc-grid h2 {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		margin: var(--space-3) 0;
	}

	.toc {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.toc li {
		display: grid;
		grid-template-columns: 64px 1fr;
		gap: var(--space-5);
		padding: var(--space-5);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		transition: all var(--dur-base) var(--ease-out);
	}
	.toc li:hover {
		border-color: var(--border-gold);
		transform: translateX(4px);
	}
	.toc .n {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		font-weight: var(--weight-semibold);
		line-height: 1;
	}
	.toc .t {
		font-size: var(--text-md);
		color: var(--ink-100);
		font-family: var(--font-display);
	}
</style>
