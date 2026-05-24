<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cx } from '$lib/utils/classes.js';

	type Variant = 'default' | 'gold' | 'success' | 'danger' | 'info' | 'warning' | 'outline';

	type Props = {
		variant?: Variant;
		size?: 'sm' | 'md';
		children?: Snippet;
		class?: string;
	};

	let { variant = 'default', size = 'md', children, class: className }: Props = $props();
</script>

<span class={cx('badge', `badge-${variant}`, `badge-${size}`, className)}>
	{@render children?.()}
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		padding: 4px var(--space-3);
		border-radius: var(--radius-full);
		font-family: var(--font-body);
		font-size: var(--text-2xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		line-height: 1;
		white-space: nowrap;
		border: 1px solid transparent;
	}
	.badge-sm {
		padding: 3px var(--space-2);
		font-size: 10px;
	}
	.badge-default {
		background: var(--surface-3);
		color: var(--ink-200);
	}
	.badge-gold {
		background: linear-gradient(135deg, rgba(245, 208, 138, 0.16), rgba(176, 131, 47, 0.08));
		color: var(--gold-300);
		border-color: var(--border-gold);
	}
	.badge-success {
		background: var(--success-bg);
		color: var(--success);
	}
	.badge-danger {
		background: var(--danger-bg);
		color: var(--danger);
	}
	.badge-info {
		background: var(--info-bg);
		color: var(--info);
	}
	.badge-warning {
		background: var(--warning-bg);
		color: var(--warning);
	}
	.badge-outline {
		background: transparent;
		color: var(--ink-200);
		border-color: var(--border-strong);
	}
</style>
