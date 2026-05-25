<script lang="ts">
	import { toasts } from '$lib/stores/toast.svelte.js';
	import { IconCheck, IconAlertTriangle, IconInfoCircle, IconX } from '@tabler/icons-svelte';
	import { fly } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';

	const iconMap = {
		success: IconCheck,
		error: IconAlertTriangle,
		warning: IconAlertTriangle,
		info: IconInfoCircle,
		default: IconInfoCircle
	};
</script>

<div class="toast-stack" aria-live="polite" aria-atomic="true">
	{#each toasts.items as t (t.id)}
		{@const IconComp = iconMap[t.variant]}
		<div
			class="toast toast-{t.variant}"
			role="status"
			transition:fly={{ x: 32, y: 0, duration: 280, easing: cubicOut }}
		>
			<div class="icon" aria-hidden="true">
				<IconComp size={18} />
			</div>
			<div class="body">
				<p class="title">{t.title}</p>
				{#if t.description}<p class="desc">{t.description}</p>{/if}
			</div>
			<button class="close" type="button" aria-label="Dismiss" onclick={() => toasts.dismiss(t.id)}>
				<IconX size={14} />
			</button>
		</div>
	{/each}
</div>

<style>
	.toast-stack {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		z-index: var(--z-toast);
		pointer-events: none;
		max-width: calc(100vw - var(--space-12));
		width: 360px;
	}
	.toast {
		pointer-events: auto;
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: var(--space-3);
		align-items: flex-start;
		padding: var(--space-4) var(--space-5);
		background: var(--surface-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-elev-3);
	}
	.toast-success {
		border-left: 3px solid var(--success);
	}
	.toast-error {
		border-left: 3px solid var(--danger);
	}
	.toast-warning {
		border-left: 3px solid var(--warning);
	}
	.toast-info {
		border-left: 3px solid var(--info);
	}
	.icon {
		width: 28px;
		height: 28px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}
	.toast-success .icon {
		background: var(--success-bg);
		color: var(--success);
	}
	.toast-error .icon {
		background: var(--danger-bg);
		color: var(--danger);
	}
	.toast-warning .icon {
		background: var(--warning-bg);
		color: var(--warning);
	}
	.toast-info .icon,
	.toast-default .icon {
		background: var(--info-bg);
		color: var(--info);
	}
	.body {
		min-width: 0;
	}
	.title {
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
		line-height: var(--leading-snug);
	}
	.desc {
		font-size: var(--text-xs);
		color: var(--ink-300);
		margin-top: 2px;
	}
	.close {
		width: 24px;
		height: 24px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		color: var(--ink-400);
		flex-shrink: 0;
	}
	.close:hover {
		color: var(--ink-100);
		background: var(--surface-3);
	}

	@media (--bp-sm-down) {
		.toast-stack {
			left: var(--space-4);
			right: var(--space-4);
			bottom: var(--space-4);
			width: auto;
			max-width: none;
		}
	}
</style>
