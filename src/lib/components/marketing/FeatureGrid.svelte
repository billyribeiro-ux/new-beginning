<script lang="ts">
	import type { Icon as IconComponent } from '@tabler/icons-svelte';
	import { stagger } from '$lib/animations/attachments.js';

	type Feature = { icon: IconComponent; title: string; description: string };
	type Props = { features: Feature[]; columns?: 2 | 3; class?: string };
	let { features, columns = 3, class: className }: Props = $props();
</script>

<div class="grid cols-{columns} {className ?? ''}" {@attach stagger({ stagger: 0.06 })}>
	{#each features as f (f.title)}
		{@const Icon = f.icon}
		<article class="card">
			<div class="icon-wrap">
				<Icon size={22} />
			</div>
			<h3>{f.title}</h3>
			<p>{f.description}</p>
		</article>
	{/each}
</div>

<style>
	.grid {
		display: grid;
		gap: var(--space-5);
		grid-template-columns: 1fr;
	}
	@media (--bp-md) {
		.grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
	@media (--bp-lg) {
		.cols-3 {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.card {
		position: relative;
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		transition:
			background var(--dur-base) var(--ease-out),
			border-color var(--dur-base) var(--ease-out),
			transform var(--dur-base) var(--ease-out);
	}
	.card:hover {
		background: var(--surface-2);
		border-color: var(--border-gold);
		transform: translateY(-2px);
	}
	.icon-wrap {
		width: 48px;
		height: 48px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-md);
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.16), rgba(212, 162, 76, 0.04));
		color: var(--gold-300);
		border: 1px solid var(--border-gold);
		margin-bottom: var(--space-5);
	}
	.card h3 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0 0 var(--space-3);
		line-height: var(--leading-snug);
	}
	.card p {
		color: var(--ink-300);
		font-size: var(--text-sm);
		line-height: var(--leading-relaxed);
		margin: 0;
	}
</style>
