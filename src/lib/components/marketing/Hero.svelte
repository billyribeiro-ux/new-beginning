<script lang="ts">
	import {
		IconArrowRight,
		IconStarFilled,
		IconChartCandle,
		IconBolt,
		IconShieldCheck
	} from '@tabler/icons-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import {
		fadeUp,
		splitReveal,
		parallax,
		cursorGlow,
		magnetic
	} from '$lib/animations/attachments.js';
</script>

<section class="hero" {@attach cursorGlow()}>
	<div class="bg-layer" {@attach parallax({ speed: 0.18 })}>
		<div class="grid-overlay"></div>
		<div class="halo halo-1"></div>
		<div class="halo halo-2"></div>
	</div>

	<div class="hero-inner container">
		<div class="hero-content">
			<div class="trust-row" {@attach fadeUp({ y: 16, delay: 0 })}>
				<Badge variant="gold">
					<IconStarFilled size={12} />
					Trusted by 14,200+ traders
				</Badge>
				<span class="trust-note">Verified setups · Live trading room · 14-day guarantee</span>
			</div>

			<h1 class="headline" {@attach splitReveal({ stagger: 0.045, delay: 0.1 })}>
				Engineered tooling for traders who run a real process.
			</h1>

			<p class="sub" {@attach fadeUp({ y: 18, delay: 0.4 })}>
				TradeFlex Trading builds the indicators, education, and live-room cadence used by serious
				day traders and options operators. No hype. No signals. Just an opinionated, battle-tested
				stack you can deploy on Monday morning.
			</p>

			<div class="ctas" {@attach fadeUp({ y: 18, delay: 0.55 })}>
				<span {@attach magnetic({ strength: 0.18 })}>
					<Button variant="primary" size="xl" href="/subscription">
						Join the Day Trading desk
						{#snippet iconRight()}<IconArrowRight size={18} />{/snippet}
					</Button>
				</span>
				<Button variant="gold-outline" size="xl" href="/free-guide"
					>Get the Free Greeks Guide</Button
				>
			</div>

			<ul class="hero-points" {@attach fadeUp({ y: 14, delay: 0.7 })}>
				<li>
					<span class="dot"><IconChartCandle size={14} /></span>Live setups on ES · NQ · CL · GC
				</li>
				<li><span class="dot"><IconBolt size={14} /></span>Sub-millisecond alerts, repaint-free</li>
				<li><span class="dot"><IconShieldCheck size={14} /></span>14-day no-questions refund</li>
			</ul>
		</div>

		<div class="hero-visual" {@attach fadeUp({ y: 30, delay: 0.45 })}>
			<div class="chart-card">
				<div class="chart-header">
					<div>
						<p class="ticker">ES · M1</p>
						<p class="price">4,892.<span>75</span></p>
					</div>
					<div class="chart-meta">
						<Badge variant="success">+1.84%</Badge>
						<span class="vol">VOL 1.2M</span>
					</div>
				</div>
				<div class="chart-body">
					<div class="candles" aria-hidden="true">
						{#each Array(28) as _, i (i)}
							{@const up = [0, 1, 4, 5, 6, 9, 10, 13, 14, 16, 18, 20, 22, 24, 25, 27].includes(i)}
							{@const ht = 18 + ((i * 13) % 70)}
							{@const top = 10 + ((i * 7) % 30)}
							<span class="candle" class:up class:down={!up} style:--h="{ht}%" style:--t="{top}%"
							></span>
						{/each}
					</div>
					<div class="band band-top"></div>
					<div class="band band-bottom"></div>
					<div class="label-top">Revolution Ranger · upper rail</div>
					<div class="label-bottom">Revolution Ranger · lower rail</div>
				</div>
				<div class="chart-footer">
					<span>Range compression detected</span>
					<Badge variant="gold">Setup live</Badge>
				</div>
			</div>
		</div>
	</div>
</section>

<style>
	.hero {
		position: relative;
		isolation: isolate;
		overflow: hidden;
		padding-block: clamp(4rem, 8vw, 8rem);
		--glow-x: 50%;
		--glow-y: 30%;
	}
	.hero::before {
		content: '';
		position: absolute;
		inset: 0;
		background: radial-gradient(
			600px circle at var(--glow-x) var(--glow-y),
			rgba(232, 182, 96, 0.08),
			transparent 60%
		);
		pointer-events: none;
		z-index: 0;
		transition: background var(--dur-base) var(--ease-out);
	}
	.bg-layer {
		position: absolute;
		inset: -20% -10%;
		z-index: -1;
		pointer-events: none;
	}
	.grid-overlay {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgba(232, 182, 96, 0.04) 1px, transparent 1px),
			linear-gradient(90deg, rgba(232, 182, 96, 0.04) 1px, transparent 1px);
		background-size: 64px 64px;
		mask-image: radial-gradient(ellipse at 50% 30%, black 30%, transparent 75%);
		-webkit-mask-image: radial-gradient(ellipse at 50% 30%, black 30%, transparent 75%);
	}
	.halo {
		position: absolute;
		border-radius: 50%;
		filter: blur(80px);
		pointer-events: none;
	}
	.halo-1 {
		top: -20%;
		right: -10%;
		width: 520px;
		height: 520px;
		background: radial-gradient(circle, rgba(232, 182, 96, 0.18), transparent 70%);
	}
	.halo-2 {
		bottom: -30%;
		left: -10%;
		width: 620px;
		height: 620px;
		background: radial-gradient(circle, rgba(176, 131, 47, 0.12), transparent 70%);
	}

	.hero-inner {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 5vw, 4.5rem);
		align-items: center;
		position: relative;
		z-index: 1;
	}
	@media (--bp-lg) {
		.hero-inner {
			grid-template-columns: 1.05fr 1fr;
		}
	}

	.trust-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}
	.trust-note {
		font-size: var(--text-xs);
		color: var(--ink-400);
		letter-spacing: var(--tracking-wide);
	}

	.headline {
		font-family: var(--font-display);
		font-size: clamp(2.5rem, 1.8rem + 4.5vw, 5.5rem);
		line-height: 1;
		letter-spacing: -0.03em;
		color: var(--ink-100);
		margin-top: var(--space-6);
		margin-bottom: 0;
		font-weight: var(--weight-semibold);
		text-wrap: balance;
	}

	.sub {
		margin-top: var(--space-6);
		max-width: 56ch;
		font-size: var(--text-md);
		line-height: var(--leading-relaxed);
		color: var(--ink-300);
	}

	.ctas {
		margin-top: var(--space-8);
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.hero-points {
		margin-top: var(--space-8);
		list-style: none;
		display: flex;
		gap: var(--space-6);
		flex-wrap: wrap;
	}
	.hero-points li {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-xs);
		color: var(--ink-300);
	}
	.dot {
		display: inline-flex;
		width: 24px;
		height: 24px;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		color: var(--gold-400);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
	}

	/* --- Chart card visual --- */
	.hero-visual {
		position: relative;
	}
	.chart-card {
		position: relative;
		padding: var(--space-6);
		/* 90–95% opaque gradient already; blur is a polish layer. */
		background: linear-gradient(160deg, rgba(31, 31, 38, 0.9), rgba(17, 17, 20, 0.95));
		border: 1px solid var(--border-default);
		border-radius: var(--radius-2xl);
		box-shadow:
			var(--shadow-elev-4),
			inset 0 1px 0 rgba(255, 255, 255, 0.04);
		overflow: hidden;
	}
	@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)) {
		.chart-card {
			backdrop-filter: blur(20px);
			-webkit-backdrop-filter: blur(20px);
		}
	}
	.chart-card::before {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.05), transparent 60%);
		pointer-events: none;
	}
	.chart-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-3);
		position: relative;
		z-index: 1;
	}
	.ticker {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--ink-400);
		letter-spacing: var(--tracking-wider);
		margin: 0;
	}
	.price {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 4px 0 0;
		line-height: 1;
	}
	.price span {
		color: var(--ink-400);
		font-weight: var(--weight-regular);
		font-size: 0.7em;
	}
	.chart-meta {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: var(--space-2);
	}
	.vol {
		font-family: var(--font-mono);
		font-size: var(--text-2xs);
		color: var(--ink-400);
	}

	.chart-body {
		position: relative;
		height: 220px;
		margin-top: var(--space-5);
		border-radius: var(--radius-md);
	}
	.candles {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: flex-end;
		gap: 4px;
	}
	.candle {
		flex: 1;
		height: var(--h);
		margin-top: var(--t);
		border-radius: 2px;
		opacity: 0.85;
	}
	.candle.up {
		background: linear-gradient(180deg, #5dbb78, #3d8e54);
	}
	.candle.down {
		background: linear-gradient(180deg, #d96868, #a04646);
	}

	.band {
		position: absolute;
		left: 0;
		right: 0;
		height: 1px;
		background: var(--gold-500);
		opacity: 0.6;
	}
	.band-top {
		top: 18%;
	}
	.band-bottom {
		bottom: 18%;
	}

	.label-top,
	.label-bottom {
		position: absolute;
		right: 0;
		padding: 2px 8px;
		background: rgba(212, 162, 76, 0.18);
		color: var(--gold-300);
		font-family: var(--font-mono);
		font-size: 10px;
		letter-spacing: var(--tracking-wider);
		border-radius: var(--radius-xs);
	}
	/* Both labels sit ABOVE their respective rail lines. The label's
	 * bottom edge rests just above the gold line, never bisecting
	 * it. Anchor each label to its rail's y-coordinate (top: 18%
	 * for the upper rail, bottom: 82% — i.e. 100% − 18% — for the
	 * lower rail measured from the top), then translate up by 100%
	 * of the label's own height plus a 4px visual gap. The
	 * percentage portion self-corrects for font-size / padding /
	 * line-height changes; the px portion is the deliberate gap. */
	.label-top,
	.label-bottom {
		transform: translateY(calc(-100% - 4px));
	}
	.label-top {
		top: 18%;
	}
	.label-bottom {
		top: 82%;
	}

	.chart-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: var(--space-5);
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-default);
		font-size: var(--text-xs);
		color: var(--ink-300);
		position: relative;
		z-index: 1;
	}
</style>
