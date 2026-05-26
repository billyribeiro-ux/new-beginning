<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';
	import { cx } from '$lib/utils/classes.js';

	type Props = HTMLTextareaAttributes & {
		label?: string;
		hint?: string;
		error?: string;
		value?: string;
		class?: string;
	};

	let {
		label,
		hint,
		error,
		value = $bindable(''),
		class: className,
		id,
		rows = 5,
		...rest
	}: Props = $props();

	const reactiveId = $derived(id ?? `ta-${Math.random().toString(36).slice(2, 9)}`);
</script>

<div class={cx('field', error && 'has-error', className)}>
	{#if label}
		<label for={reactiveId}>{label}</label>
	{/if}
	<textarea id={reactiveId} bind:value {rows} aria-invalid={!!error} {...rest}></textarea>
	{#if error}
		<p class="msg msg-error" role="alert">{error}</p>
	{:else if hint}
		<p class="msg msg-hint">{hint}</p>
	{/if}
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	label {
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		color: var(--ink-300);
	}
	textarea {
		width: 100%;
		padding: var(--space-3) var(--space-4);
		font-family: inherit;
		font-size: var(--text-base);
		color: var(--ink-100);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		resize: vertical;
		min-height: 120px;
		line-height: var(--leading-relaxed);
		transition:
			border-color var(--dur-base) var(--ease-out),
			background var(--dur-base) var(--ease-out),
			box-shadow var(--dur-base) var(--ease-out);
	}
	textarea::placeholder {
		color: var(--ink-400);
	}
	textarea:focus {
		outline: none;
		border-color: var(--gold-500);
		background: var(--surface-2);
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.18);
	}
	.has-error textarea {
		border-color: var(--danger);
	}
	.msg {
		font-size: var(--text-xs);
		line-height: var(--leading-snug);
	}
	.msg-hint {
		color: var(--ink-400);
	}
	.msg-error {
		color: var(--danger);
	}
</style>
