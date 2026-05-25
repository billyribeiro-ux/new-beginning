<script lang="ts">
	import { formatPriceParts } from '$lib/utils/money.js';

	type Props = {
		cents: number;
		originalCents?: number;
		suffix?: string;
		size?: 'sm' | 'md' | 'lg' | 'xl';
		align?: 'left' | 'right' | 'center';
	};
	let { cents, originalCents, suffix, size = 'md', align = 'left' }: Props = $props();
	const parts = $derived(formatPriceParts(cents));
	const originalParts = $derived(originalCents ? formatPriceParts(originalCents) : null);
</script>

<span class="price size-{size} align-{align}">
	{#if originalParts}
		<s class="strike">{originalParts.symbol}{originalParts.whole}</s>
	{/if}
	<span class="amount">
		<span class="symbol">{parts.symbol}</span>
		<span class="whole">{parts.whole}</span>
		{#if parts.fraction && parts.fraction !== '00'}
			<span class="fraction">.{parts.fraction}</span>
		{/if}
	</span>
	{#if suffix}<span class="suffix">{suffix}</span>{/if}
</span>

<style>
	.price {
		display: inline-flex;
		align-items: baseline;
		gap: var(--space-2);
		font-family: var(--font-display);
		color: var(--ink-100);
		font-weight: var(--weight-semibold);
		line-height: 1;
		letter-spacing: var(--tracking-tight);
	}
	.align-center {
		justify-content: center;
	}
	.align-right {
		justify-content: flex-end;
	}

	.amount {
		display: inline-flex;
		align-items: baseline;
	}
	.symbol {
		font-size: 0.55em;
		color: var(--ink-300);
		margin-inline-end: 2px;
		font-weight: var(--weight-regular);
	}
	.whole {
		font-feature-settings: 'tnum';
	}
	.fraction {
		font-size: 0.55em;
		color: var(--ink-300);
		font-weight: var(--weight-regular);
	}
	.strike {
		font-family: var(--font-body);
		color: var(--ink-400);
		font-size: 0.55em;
		font-weight: var(--weight-regular);
	}
	.suffix {
		font-family: var(--font-body);
		font-size: 0.4em;
		font-weight: var(--weight-medium);
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
	}

	.size-sm {
		font-size: var(--text-xl);
	}
	.size-md {
		font-size: var(--text-3xl);
	}
	.size-lg {
		font-size: var(--text-5xl);
	}
	.size-xl {
		font-size: var(--text-7xl);
	}
</style>
