<script lang="ts">
	import { IconTrendingUp, IconTrendingDown, IconActivity } from '@tabler/icons-svelte';
	import { numberTicker } from '$lib/animations/attachments.js';

	type IconLike = typeof IconActivity;
	type Props = {
		label: string;
		value: number;
		formatter?: (n: number) => string;
		prefix?: string;
		suffix?: string;
		deltaPct?: number;
		spark?: number[];
		icon?: IconLike;
		hint?: string;
	};
	let {
		label,
		value,
		formatter,
		prefix,
		suffix,
		deltaPct,
		spark = [],
		icon: Icon,
		hint
	}: Props = $props();

	const fmt = (n: number) => {
		const base = (formatter ?? ((x: number) => Math.round(x).toLocaleString()))(n);
		return `${prefix ?? ''}${base}${suffix ?? ''}`;
	};

	const sparkPath = $derived.by(() => {
		if (spark.length < 2) return [];
		const max = Math.max(...spark);
		const min = Math.min(...spark);
		const range = max - min || 1;
		return spark.map((v, i) => ({
			left: (i / (spark.length - 1)) * 100,
			top: 100 - ((v - min) / range) * 100
		}));
	});
</script>

<article class="stat-card">
	<header>
		<span class="label">{label}</span>
		{#if Icon}<span class="icon"><Icon size={16} /></span>{/if}
	</header>
	<div class="value" {@attach numberTicker({ to: value, format: fmt })}>{fmt(0)}</div>

	<footer class="foot">
		{#if deltaPct !== undefined}
			<span class="delta" class:is-up={deltaPct > 0} class:is-down={deltaPct < 0}>
				{#if deltaPct > 0}<IconTrendingUp size={14} />{:else if deltaPct < 0}<IconTrendingDown
						size={14}
					/>{/if}
				{deltaPct > 0 ? '+' : ''}{deltaPct.toFixed(1)}%
			</span>
			{#if hint}<span class="hint">{hint}</span>{/if}
		{:else if hint}
			<span class="hint">{hint}</span>
		{/if}
	</footer>

	{#if sparkPath.length > 1}
		<div class="spark" aria-hidden="true">
			{#each sparkPath as p, i (i)}
				<span class="dot" style:left="{p.left}%" style:top="{p.top}%"></span>
			{/each}
			{#each sparkPath.slice(1) as p, i (i)}
				{@const prev = sparkPath[i]}
				{#if prev}
					{@const dx = p.left - prev.left}
					{@const dy = p.top - prev.top}
					{@const len = Math.sqrt(dx * dx + dy * dy)}
					{@const ang = (Math.atan2(dy, dx) * 180) / Math.PI}
					<span
						class="line"
						style:left="{prev.left}%"
						style:top="{prev.top}%"
						style:width="{len}%"
						style:transform="rotate({ang}deg)"
					></span>
				{/if}
			{/each}
		</div>
	{/if}
</article>

<style>
	.stat-card {
		/* Container-relative: in a tight dashboard column the sparkline
		 * would overlap the value; container queries hide it cleanly
		 * regardless of viewport width. */
		container-type: inline-size;
		container-name: stat-card;

		position: relative;
		padding: var(--space-5) var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		min-height: 132px;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		overflow: hidden;
		transition: all var(--dur-base) var(--ease-out);
	}
	.stat-card:hover {
		border-color: var(--border-gold);
	}
	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.label {
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.icon {
		display: inline-flex;
		width: 28px;
		height: 28px;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--gold-400);
	}
	.value {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		line-height: 1;
		letter-spacing: var(--tracking-tight);
		font-feature-settings: 'tnum';
	}
	.foot {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--text-xs);
	}
	.delta {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		padding: 2px var(--space-2);
		border-radius: var(--radius-full);
		font-weight: var(--weight-semibold);
		background: var(--surface-2);
		color: var(--ink-300);
	}
	.delta.is-up {
		background: var(--success-bg);
		color: var(--success);
	}
	.delta.is-down {
		background: var(--danger-bg);
		color: var(--danger);
	}
	.hint {
		color: var(--ink-400);
	}

	.spark {
		position: absolute;
		bottom: var(--space-3);
		right: var(--space-5);
		width: 100px;
		height: 36px;
		opacity: 0.6;
	}
	/* Below ~15rem (240px) the sparkline can't fit alongside the value
	 * without overlapping; hide it. The card still tells the same
	 * story via the delta percent in the footer. */
	@container stat-card (max-width: 15rem) {
		.spark {
			display: none;
		}
	}
	.dot {
		position: absolute;
		width: 4px;
		height: 4px;
		background: var(--gold-400);
		border-radius: 50%;
		transform: translate(-50%, -50%);
	}
	.line {
		position: absolute;
		height: 1px;
		background: var(--gold-500);
		transform-origin: left center;
		opacity: 0.7;
	}
</style>
