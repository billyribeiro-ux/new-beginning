<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { IconCheck } from '@tabler/icons-svelte';

	type Props = Omit<HTMLInputAttributes, 'type' | 'checked'> & {
		label?: string;
		checked?: boolean;
		children?: Snippet;
	};

	let { label, checked = $bindable(false), id, children, ...rest }: Props = $props();
	const reactiveId = $derived(id ?? `cb-${Math.random().toString(36).slice(2, 9)}`);
</script>

<label class="cb" for={reactiveId}>
	<input id={reactiveId} type="checkbox" bind:checked {...rest} />
	<span class="box" aria-hidden="true">
		{#if checked}<IconCheck size={14} stroke={3} />{/if}
	</span>
	<span class="label-text">
		{#if children}{@render children()}{:else if label}{label}{/if}
	</span>
</label>

<style>
	.cb {
		display: inline-flex;
		align-items: flex-start;
		gap: var(--space-3);
		cursor: pointer;
		user-select: none;
		font-size: var(--text-sm);
		color: var(--ink-200);
		line-height: var(--leading-snug);
	}

	input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}

	.box {
		flex-shrink: 0;
		width: 20px;
		height: 20px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-1);
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-xs);
		color: var(--surface-0);
		transition: all var(--dur-fast) var(--ease-out);
	}

	input:focus-visible + .box {
		outline: 2px solid var(--gold-400);
		outline-offset: 2px;
	}

	input:checked + .box {
		background: var(--gradient-gold);
		border-color: transparent;
	}

	.cb:hover .box {
		border-color: var(--gold-500);
	}

	.label-text {
		padding-top: 1px;
	}
	.label-text:empty {
		display: none;
	}
	.label-text :global(a) {
		color: var(--gold-300);
		text-decoration: underline;
	}
</style>
