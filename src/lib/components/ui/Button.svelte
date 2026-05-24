<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cx } from '$lib/utils/classes.js';

	type Variant = 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger' | 'gold-outline';
	type Size = 'sm' | 'md' | 'lg' | 'xl';

	type Props = {
		variant?: Variant;
		size?: Size;
		fullWidth?: boolean;
		loading?: boolean;
		disabled?: boolean;
		iconLeft?: Snippet;
		iconRight?: Snippet;
		children?: Snippet;
		class?: string;
		href?: string;
		target?: '_self' | '_blank' | '_parent' | '_top';
		rel?: string;
		type?: 'button' | 'submit' | 'reset';
		onclick?: (event: MouseEvent) => void;
		ariaLabel?: string;
	};

	let {
		variant = 'primary',
		size = 'md',
		fullWidth = false,
		loading = false,
		disabled = false,
		iconLeft,
		iconRight,
		children,
		class: className,
		href,
		target,
		rel,
		type = 'button',
		onclick,
		ariaLabel
	}: Props = $props();

	const classes = $derived(
		cx(
			'btn',
			`btn-${variant}`,
			`btn-${size}`,
			fullWidth && 'btn-full',
			loading && 'is-loading',
			className
		)
	);
</script>

{#if href}
	<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- Button accepts pre-resolved internal paths, hash anchors, and external URLs; no base path is configured. -->
	<a {href} {target} {rel} class={classes} aria-busy={loading} aria-label={ariaLabel} {onclick}>
		{#if iconLeft}<span class="icon">{@render iconLeft()}</span>{/if}
		<span class="label">{@render children?.()}</span>
		{#if iconRight}<span class="icon">{@render iconRight()}</span>{/if}
		{#if loading}<span class="spinner" aria-hidden="true"></span>{/if}
	</a>
{:else}
	<button
		{type}
		class={classes}
		aria-busy={loading}
		aria-label={ariaLabel}
		disabled={loading || disabled}
		{onclick}
	>
		{#if iconLeft}<span class="icon">{@render iconLeft()}</span>{/if}
		<span class="label">{@render children?.()}</span>
		{#if iconRight}<span class="icon">{@render iconRight()}</span>{/if}
		{#if loading}<span class="spinner" aria-hidden="true"></span>{/if}
	</button>
{/if}

<style>
	.btn {
		--bg: var(--surface-2);
		--bg-hover: var(--surface-3);
		--fg: var(--ink-100);
		--border: var(--border-default);
		--shadow: var(--shadow-elev-1);
		--ring: transparent;

		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding-inline: var(--space-5);
		height: var(--h, 44px);
		font-family: var(--font-body);
		font-weight: var(--weight-semibold);
		font-size: var(--text-sm);
		letter-spacing: var(--tracking-wide);
		line-height: 1;
		color: var(--fg);
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: var(--radius-full);
		box-shadow:
			var(--shadow),
			0 0 0 0 var(--ring);
		cursor: pointer;
		text-decoration: none;
		user-select: none;
		white-space: nowrap;
		transition:
			background var(--dur-base) var(--ease-out),
			color var(--dur-base) var(--ease-out),
			border-color var(--dur-base) var(--ease-out),
			transform var(--dur-base) var(--ease-out),
			box-shadow var(--dur-base) var(--ease-out);
		position: relative;
		overflow: hidden;
	}

	.btn:hover {
		background: var(--bg-hover);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}

	.btn:active {
		transform: translateY(0);
	}

	.btn:disabled,
	.btn[aria-busy='true'] {
		opacity: 0.6;
		cursor: not-allowed;
		transform: none;
	}

	.btn-primary {
		--bg: var(--gradient-gold);
		--bg-hover: linear-gradient(135deg, #f5d08a, #e8b660 50%, #d4a24c);
		--fg: #1a1206;
		--border: transparent;
		--shadow: 0 8px 24px rgba(212, 162, 76, 0.28), inset 0 -1px 0 rgba(0, 0, 0, 0.18);
	}
	.btn-primary:hover {
		box-shadow:
			0 14px 36px rgba(232, 182, 96, 0.4),
			inset 0 -1px 0 rgba(0, 0, 0, 0.18);
	}

	.btn-secondary {
		--bg: var(--surface-3);
		--bg-hover: var(--surface-4);
		--fg: var(--ink-100);
		--border: var(--border-default);
	}

	.btn-ghost {
		--bg: transparent;
		--bg-hover: var(--surface-2);
		--fg: var(--ink-200);
		--border: transparent;
	}

	.btn-outline {
		--bg: transparent;
		--bg-hover: var(--surface-2);
		--fg: var(--ink-100);
		--border: var(--border-strong);
	}

	.btn-gold-outline {
		--bg: transparent;
		--bg-hover: rgba(232, 182, 96, 0.08);
		--fg: var(--gold-300);
		--border: var(--border-gold);
	}
	.btn-gold-outline:hover {
		--fg: var(--gold-200);
		box-shadow: var(--glow-gold);
	}

	.btn-danger {
		--bg: var(--danger);
		--bg-hover: #c25656;
		--fg: #1a0808;
		--border: transparent;
	}

	.btn-sm {
		--h: 36px;
		font-size: var(--text-xs);
		padding-inline: var(--space-4);
	}
	.btn-md {
		--h: 44px;
	}
	.btn-lg {
		--h: 52px;
		font-size: var(--text-base);
		padding-inline: var(--space-6);
	}
	.btn-xl {
		--h: 60px;
		font-size: var(--text-md);
		padding-inline: var(--space-8);
	}

	.btn-full {
		width: 100%;
	}

	.icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.label {
		display: inline-block;
	}
	.label:empty {
		display: none;
	}

	.is-loading .label,
	.is-loading .icon {
		opacity: 0.4;
	}

	.spinner {
		position: absolute;
		inset: 0;
		margin: auto;
		width: 18px;
		height: 18px;
		border: 2px solid currentColor;
		border-top-color: transparent;
		border-radius: var(--radius-full);
		animation: spin 0.7s linear infinite;
	}
</style>
