<script lang="ts">
	import LogoMark from './LogoMark.svelte';

	type Props = { href?: string; size?: number; tagline?: boolean };
	let { href = '/', size = 36, tagline = false }: Props = $props();
</script>

<a class="logo" {href} aria-label="TradeFlex Trading — home">
	<LogoMark {size} />
	<span class="text">
		<span class="word">
			Trade<span class="flex">Flex</span>
		</span>
		{#if tagline}<span class="tag">Trading · Est. 2026</span>{/if}
	</span>
</a>

<style>
	.logo {
		display: inline-flex;
		align-items: center;
		gap: var(--space-3);
		text-decoration: none;
		color: var(--ink-100);
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		letter-spacing: var(--tracking-tight);
		transition: transform var(--dur-base) var(--ease-out);
	}
	.logo:hover {
		transform: translateY(-1px);
	}
	.text {
		display: inline-flex;
		flex-direction: column;
		line-height: 1;
	}
	.word {
		font-size: var(--text-xl);
		letter-spacing: -0.01em;
	}
	.flex {
		background: var(--gradient-text-gold);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		color: transparent;
	}
	.tag {
		margin-top: 4px;
		font-family: var(--font-body);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-regular);
	}
</style>
