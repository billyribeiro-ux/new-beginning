<script lang="ts">
	import IconStarFilled from '@tabler/icons-svelte/icons/star-filled';
	import IconArrowLeft from '@tabler/icons-svelte/icons/arrow-left';
	import IconArrowRight from '@tabler/icons-svelte/icons/arrow-right';
	import IconQuote from '@tabler/icons-svelte/icons/quote';
	import { TESTIMONIALS } from '$lib/data/testimonials.js';
	import { fadeUp } from '$lib/animations/attachments.js';

	let index = $state(0);
	const visible = $derived(
		TESTIMONIALS.slice(index, index + 3).concat(
			index + 3 > TESTIMONIALS.length ? TESTIMONIALS.slice(0, index + 3 - TESTIMONIALS.length) : []
		)
	);

	function prev() {
		index = (index - 1 + TESTIMONIALS.length) % TESTIMONIALS.length;
	}
	function next() {
		index = (index + 1) % TESTIMONIALS.length;
	}
</script>

<div class="carousel" {@attach fadeUp({ y: 24 })}>
	<div class="grid">
		{#each visible as t (t.id + '-' + index)}
			<article class="card">
				<div class="quote-mark" aria-hidden="true"><IconQuote size={28} /></div>
				<div class="rating" aria-label="{t.rating} out of 5">
					{#each Array(t.rating) as _, i (i)}
						<IconStarFilled size={14} />
					{/each}
				</div>
				<p class="quote">{t.quote}</p>
				<footer class="who">
					<div class="avatar" style:background={t.avatarColor} aria-hidden="true">{t.initials}</div>
					<div class="meta">
						<p class="name">{t.name}</p>
						<p class="role">{t.role}</p>
					</div>
					{#if t.context}<span class="context">{t.context}</span>{/if}
				</footer>
			</article>
		{/each}
	</div>

	<div class="controls">
		<button type="button" aria-label="Previous testimonials" onclick={prev}
			><IconArrowLeft size={18} /></button
		>
		<p class="counter">
			<span>{index + 1}</span> / {TESTIMONIALS.length}
		</p>
		<button type="button" aria-label="Next testimonials" onclick={next}
			><IconArrowRight size={18} /></button
		>
	</div>
</div>

<style>
	.carousel {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}
	.grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (--bp-md) {
		.grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (--bp-xl) {
		.grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.card {
		position: relative;
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		overflow: hidden;
		transition: all var(--dur-base) var(--ease-out);
	}
	.card:hover {
		border-color: var(--border-gold);
		transform: translateY(-2px);
	}
	.quote-mark {
		position: absolute;
		top: var(--space-5);
		right: var(--space-5);
		color: var(--gold-700);
		opacity: 0.6;
	}
	.rating {
		display: inline-flex;
		gap: 2px;
		color: var(--gold-400);
	}
	.quote {
		font-family: var(--font-display);
		font-size: var(--text-md);
		line-height: var(--leading-relaxed);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-regular);
	}
	.who {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: var(--space-3);
		align-items: center;
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-default);
	}
	.avatar {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-full);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--surface-0);
		font-weight: var(--weight-bold);
		font-size: var(--text-sm);
	}
	.name {
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
	}
	.role {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
	}
	.context {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		padding: 2px 8px;
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-full);
		white-space: nowrap;
	}

	.controls {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-4);
	}
	.controls button {
		width: 44px;
		height: 44px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-200);
		transition: all var(--dur-fast) var(--ease-out);
	}
	.controls button:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
	.counter {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
		letter-spacing: var(--tracking-wider);
	}
	.counter span {
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
	}
</style>
