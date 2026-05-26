<script lang="ts">
	import IconTrendingUp3 from '@tabler/icons-svelte/icons/trending-up-3';
	type Props = { size?: number };
	let { size = 36 }: Props = $props();
</script>

<span class="logo-mark" style:--sz="{size}px" aria-hidden="true">
	<span class="ring"></span>
	<span class="icon"><IconTrendingUp3 size={Math.round(size * 0.5)} stroke={2.4} /></span>
</span>

<style>
	.logo-mark {
		position: relative;
		display: inline-flex;
		width: var(--sz);
		height: var(--sz);
		flex-shrink: 0;
	}
	.ring {
		position: absolute;
		inset: 0;
		border-radius: var(--radius-md);
		background: var(--gradient-gold);
		box-shadow:
			0 6px 20px rgba(212, 162, 76, 0.35),
			inset 0 -1px 0 rgba(0, 0, 0, 0.2);
	}
	.ring::after {
		content: '';
		position: absolute;
		inset: 2px;
		background: var(--surface-0);
		border-radius: calc(var(--radius-md) - 2px);
	}
	.icon {
		position: relative;
		z-index: 1;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		color: var(--gold-300);
	}
</style>
