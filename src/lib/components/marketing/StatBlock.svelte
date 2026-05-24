<script lang="ts">
	import { numberTicker, stagger } from '$lib/animations/attachments.js';

	type Stat = {
		value: number;
		suffix?: string;
		prefix?: string;
		label: string;
		formatter?: (n: number) => string;
	};
	type Props = { stats: Stat[] };
	let { stats }: Props = $props();

	function fmt(s: Stat) {
		return (n: number) => `${s.prefix ?? ''}${(s.formatter ?? defaultFmt)(n)}${s.suffix ?? ''}`;
	}
	function defaultFmt(n: number) {
		if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
		if (n >= 1_000) return `${(n / 1_000).toFixed(n >= 10_000 ? 0 : 1)}k`;
		return Math.round(n).toLocaleString();
	}
</script>

<div class="stats" {@attach stagger({ stagger: 0.08, y: 18 })}>
	{#each stats as s}
		<div class="stat">
			<p class="value" {@attach numberTicker({ to: s.value, format: fmt(s) })}>0</p>
			<p class="label">{s.label}</p>
		</div>
	{/each}
</div>

<style>
	.stats {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-6);
	}
	@media (min-width: 768px) {
		.stats {
			grid-template-columns: repeat(4, 1fr);
		}
	}
	.stat {
		padding: var(--space-6) var(--space-5);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		text-align: center;
		transition: all var(--dur-base) var(--ease-out);
	}
	.stat:hover {
		border-color: var(--border-gold);
		background: var(--surface-2);
	}
	.value {
		font-family: var(--font-display);
		font-size: var(--text-5xl);
		font-weight: var(--weight-semibold);
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		color: transparent;
		line-height: 1;
		margin: 0 0 var(--space-3);
		letter-spacing: var(--tracking-tighter);
	}
	.label {
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-300);
		margin: 0;
	}
</style>
