<script lang="ts">
	import { resolve } from '$app/paths';

	type Props = { href?: string; size?: number; tagline?: boolean };
	let { href, size = 36, tagline = false }: Props = $props();

	const intrinsicW = 3163;
	const intrinsicH = 835;
	let displayWidth = $derived(Math.round((size * intrinsicW) / intrinsicH));
</script>

<a
	class="logo"
	href={resolve((href ?? '/') as unknown as '/')}
	aria-label="Revolution Trading Pros — home"
>
	<img
		src="/revolution-trading-pros-logo.png"
		alt="Revolution Trading Pros"
		width={displayWidth}
		height={size}
		style:height="{size}px"
		style:width="auto"
		decoding="async"
		fetchpriority={tagline ? 'low' : 'high'}
	/>
</a>

<style>
	.logo {
		display: inline-flex;
		align-items: center;
		text-decoration: none;
		transition: transform var(--dur-base) var(--ease-out);
	}
	.logo:hover {
		transform: translateY(-1px);
	}
	.logo img {
		display: block;
		max-width: 100%;
		object-fit: contain;
	}
</style>
