<script lang="ts">
	import type { Snippet } from 'svelte';
	import LogoWordmark from '$lib/components/media/LogoWordmark.svelte';
	import { IconArrowLeft, IconStarFilled } from '@tabler/icons-svelte';
	import { resolve } from '$app/paths';

	type Props = { children: Snippet; testimonial?: { quote: string; name: string; role: string } };
	let {
		children,
		testimonial = {
			quote:
				'Lorem ipsum dolor sit amet, consectetur adipiscing elit. The Day Trading desk gave me a process I can actually defend on a bad week.',
			name: 'Marcus A.',
			role: 'Prop Trader · 8 yrs'
		}
	}: Props = $props();
</script>

<div class="auth-shell">
	<aside class="brand-side">
		<a class="back" href={resolve('/')}><IconArrowLeft size={14} />Back to site</a>
		<LogoWordmark href="/" size={44} tagline />

		<div class="quote">
			<div class="rating">
				{#each Array(5) as _, i (i)}<IconStarFilled size={14} />{/each}
			</div>
			<p>“{testimonial.quote}”</p>
			<footer>
				<strong>{testimonial.name}</strong>
				<span>{testimonial.role}</span>
			</footer>
		</div>

		<div class="trust">
			<div><strong>14,200+</strong><span>active members</span></div>
			<div><strong>14-day</strong><span>refund window</span></div>
			<div><strong>4.9 / 5</strong><span>average rating</span></div>
		</div>
	</aside>

	<main class="form-side">
		<div class="form-shell">
			{@render children()}
		</div>
	</main>
</div>

<style>
	.auth-shell {
		min-height: 100dvh;
		display: grid;
		grid-template-columns: 1fr;
		background: var(--surface-0);
	}
	@media (--bp-lg) {
		.auth-shell {
			grid-template-columns: 1fr 1.05fr;
		}
	}

	.brand-side {
		display: none;
		position: relative;
		padding: clamp(2rem, 4vw, 4rem);
		background:
			radial-gradient(ellipse at top right, rgba(232, 182, 96, 0.16), transparent 60%),
			linear-gradient(160deg, var(--surface-1), var(--surface-0));
		border-right: 1px solid var(--border-default);
		flex-direction: column;
		justify-content: space-between;
		overflow: hidden;
	}
	.brand-side::before {
		content: '';
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgba(232, 182, 96, 0.04) 1px, transparent 1px),
			linear-gradient(90deg, rgba(232, 182, 96, 0.04) 1px, transparent 1px);
		background-size: 48px 48px;
		mask-image: radial-gradient(ellipse at top right, black 30%, transparent 80%);
		-webkit-mask-image: radial-gradient(ellipse at top right, black 30%, transparent 80%);
		pointer-events: none;
	}
	@media (--bp-lg) {
		.brand-side {
			display: flex;
		}
	}

	.back {
		position: relative;
		z-index: 1;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--ink-300);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		text-decoration: none;
		width: max-content;
	}
	.back:hover {
		color: var(--gold-300);
	}

	.quote {
		position: relative;
		z-index: 1;
		padding: var(--space-7);
		/* Opaque fallback for browsers without backdrop-filter. */
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		max-width: 520px;
	}
	@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)) {
		.quote {
			background: rgba(10, 10, 11, 0.5);
			backdrop-filter: blur(12px);
			-webkit-backdrop-filter: blur(12px);
		}
	}
	.rating {
		display: flex;
		gap: 2px;
		color: var(--gold-400);
		margin-bottom: var(--space-3);
	}
	.quote p {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		color: var(--ink-100);
		line-height: var(--leading-relaxed);
		margin: 0 0 var(--space-4);
	}
	.quote footer {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.quote strong {
		font-size: var(--text-sm);
		color: var(--ink-100);
	}
	.quote span {
		font-size: var(--text-xs);
		color: var(--ink-400);
	}

	.trust {
		position: relative;
		z-index: 1;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-4);
		padding: var(--space-5);
		/* Opaque fallback for browsers without backdrop-filter. */
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)) {
		.trust {
			background: rgba(10, 10, 11, 0.4);
			backdrop-filter: blur(12px);
			-webkit-backdrop-filter: blur(12px);
		}
	}
	.trust > div {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.trust strong {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		line-height: 1;
	}
	.trust span {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
	}

	.form-side {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: clamp(2rem, 5vw, 4rem) var(--space-4);
	}
	.form-shell {
		width: 100%;
		max-width: 520px;
	}
</style>
