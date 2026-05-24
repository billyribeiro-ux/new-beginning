<script lang="ts">
	import type { Snippet } from 'svelte';
	import { IconX } from '@tabler/icons-svelte';
	import { fade, scale } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';

	type Props = {
		open: boolean;
		title?: string;
		description?: string;
		size?: 'sm' | 'md' | 'lg';
		onclose?: () => void;
		children?: Snippet;
		footer?: Snippet;
	};

	let {
		open = $bindable(false),
		title,
		description,
		size = 'md',
		onclose,
		children,
		footer
	}: Props = $props();

	let dialogEl: HTMLDivElement | undefined = $state();

	function close() {
		open = false;
		onclose?.();
	}

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape' && open) close();
	}

	$effect(() => {
		if (open) {
			document.body.style.overflow = 'hidden';
			queueMicrotask(() => dialogEl?.focus());
		} else {
			document.body.style.overflow = '';
		}
		return () => {
			document.body.style.overflow = '';
		};
	});
</script>

<svelte:window onkeydown={onKey} />

{#if open}
	<div class="overlay" transition:fade={{ duration: 180 }}>
		<button class="backdrop" aria-label="Close dialog" onclick={close}></button>
		<div
			bind:this={dialogEl}
			class="dialog dialog-{size}"
			role="dialog"
			aria-modal="true"
			aria-labelledby={title ? 'modal-title' : undefined}
			tabindex="-1"
			transition:scale={{ duration: 220, start: 0.96, easing: cubicOut }}
		>
			<header class="dialog-header">
				<div>
					{#if title}<h3 id="modal-title">{title}</h3>{/if}
					{#if description}<p class="desc">{description}</p>{/if}
				</div>
				<button class="close" type="button" aria-label="Close" onclick={close}>
					<IconX size={18} />
				</button>
			</header>
			<div class="dialog-body">
				{@render children?.()}
			</div>
			{#if footer}
				<footer class="dialog-footer">{@render footer()}</footer>
			{/if}
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		z-index: var(--z-modal);
		display: grid;
		place-items: center;
		padding: var(--space-4);
	}
	.backdrop {
		position: absolute;
		inset: 0;
		background: var(--surface-overlay);
		backdrop-filter: blur(12px) saturate(140%);
		-webkit-backdrop-filter: blur(12px) saturate(140%);
		border: 0;
	}
	.dialog {
		position: relative;
		width: 100%;
		max-height: calc(100vh - var(--space-8));
		background: var(--surface-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-elev-4);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}
	.dialog-sm {
		max-width: 420px;
	}
	.dialog-md {
		max-width: 560px;
	}
	.dialog-lg {
		max-width: 820px;
	}

	.dialog-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-6) var(--space-6) var(--space-4);
		border-bottom: 1px solid var(--border-default);
	}
	h3 {
		font-size: var(--text-xl);
		margin: 0;
	}
	.desc {
		margin-top: var(--space-2);
		color: var(--ink-300);
		font-size: var(--text-sm);
	}
	.close {
		width: 36px;
		height: 36px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		color: var(--ink-300);
		border: 1px solid var(--border-default);
		background: var(--surface-2);
		transition: all var(--dur-fast) var(--ease-out);
	}
	.close:hover {
		color: var(--ink-100);
		background: var(--surface-3);
	}
	.dialog-body {
		padding: var(--space-6);
		overflow-y: auto;
	}
	.dialog-footer {
		display: flex;
		gap: var(--space-3);
		justify-content: flex-end;
		padding: var(--space-4) var(--space-6) var(--space-6);
		border-top: 1px solid var(--border-default);
		background: var(--surface-1);
	}
</style>
