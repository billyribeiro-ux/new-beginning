<script lang="ts">
	import type { Snippet } from 'svelte';
	type Props = {
		label: string;
		children?: Snippet;
		side?: 'top' | 'bottom';
	};
	let { label, children, side = 'top' }: Props = $props();
</script>

<span class="tt-root" data-side={side} role="presentation">
	{@render children?.()}
	<span class="tt-bubble" role="tooltip">{label}</span>
</span>

<style>
	.tt-root {
		position: relative;
		display: inline-flex;
	}
	.tt-bubble {
		position: absolute;
		left: 50%;
		transform: translateX(-50%) translateY(-8px) scale(0.96);
		background: var(--surface-elevated);
		color: var(--ink-100);
		font-size: var(--text-xs);
		padding: 6px 10px;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		white-space: nowrap;
		pointer-events: none;
		opacity: 0;
		transition:
			opacity var(--dur-fast) var(--ease-out),
			transform var(--dur-base) var(--ease-spring);
		z-index: var(--z-tooltip);
		box-shadow: var(--shadow-elev-2);
	}
	[data-side='top'] .tt-bubble {
		bottom: calc(100% + 4px);
	}
	[data-side='bottom'] .tt-bubble {
		top: calc(100% + 4px);
		transform: translateX(-50%) translateY(8px) scale(0.96);
	}
	.tt-root:hover .tt-bubble,
	.tt-root:focus-within .tt-bubble {
		opacity: 1;
		transform: translateX(-50%) translateY(0) scale(1);
	}
</style>
