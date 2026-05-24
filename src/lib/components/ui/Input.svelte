<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { cx } from '$lib/utils/classes.js';

	type Props = HTMLInputAttributes & {
		label?: string;
		hint?: string;
		error?: string;
		value?: string | number;
		class?: string;
	};

	let {
		label,
		hint,
		error,
		value = $bindable(''),
		class: className,
		id,
		...rest
	}: Props = $props();

	const reactiveId = $derived(id ?? `inp-${Math.random().toString(36).slice(2, 9)}`);
</script>

<div class={cx('field', error && 'has-error', className)}>
	{#if label}
		<label for={reactiveId}>{label}</label>
	{/if}
	<div class="input-wrap">
		<input
			id={reactiveId}
			bind:value
			aria-invalid={!!error}
			aria-describedby={error ? `${reactiveId}-err` : hint ? `${reactiveId}-hint` : undefined}
			{...rest}
		/>
	</div>
	{#if error}
		<p id="{reactiveId}-err" class="msg msg-error" role="alert">{error}</p>
	{:else if hint}
		<p id="{reactiveId}-hint" class="msg msg-hint">{hint}</p>
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

	.input-wrap {
		position: relative;
	}

	input {
		width: 100%;
		height: 48px;
		padding-inline: var(--space-4);
		font-family: inherit;
		font-size: var(--text-base);
		color: var(--ink-100);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		transition:
			border-color var(--dur-base) var(--ease-out),
			box-shadow var(--dur-base) var(--ease-out),
			background var(--dur-base) var(--ease-out);
	}

	input::placeholder {
		color: var(--ink-400);
	}

	input:hover {
		border-color: var(--border-strong);
	}

	input:focus {
		outline: none;
		border-color: var(--gold-500);
		background: var(--surface-2);
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.18);
	}

	input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.has-error input {
		border-color: var(--danger);
	}

	.has-error input:focus {
		box-shadow: 0 0 0 3px rgba(217, 104, 104, 0.2);
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
