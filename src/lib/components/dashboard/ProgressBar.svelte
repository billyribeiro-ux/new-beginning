<script lang="ts">
	type Props = { value: number; max?: number; label?: string; size?: 'sm' | 'md' };
	let { value, max = 100, label, size = 'md' }: Props = $props();
	const pct = $derived(Math.max(0, Math.min(100, (value / max) * 100)));
</script>

<div class="pb size-{size}">
	{#if label}
		<div class="head">
			<span>{label}</span>
			<span class="val">{Math.round(pct)}%</span>
		</div>
	{/if}
	<div
		class="track"
		role="progressbar"
		aria-valuenow={Math.round(pct)}
		aria-valuemin="0"
		aria-valuemax="100"
		aria-label={label}
	>
		<div class="fill" style:width="{pct}%"></div>
	</div>
</div>

<style>
	.pb {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.head {
		display: flex;
		justify-content: space-between;
		font-size: var(--text-xs);
		color: var(--ink-300);
	}
	.val {
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
	}
	.track {
		height: 6px;
		background: var(--surface-3);
		border-radius: var(--radius-full);
		overflow: hidden;
	}
	.size-sm .track {
		height: 4px;
	}
	.fill {
		height: 100%;
		background: var(--gradient-gold);
		border-radius: var(--radius-full);
		transition: width var(--dur-slower) var(--ease-out);
		box-shadow: 0 0 16px rgba(232, 182, 96, 0.4);
	}
</style>
