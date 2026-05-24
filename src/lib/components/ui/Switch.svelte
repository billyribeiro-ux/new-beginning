<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	type Props = Omit<HTMLInputAttributes, 'type' | 'checked'> & {
		checked?: boolean;
		label?: string;
		description?: string;
	};
	let {
		checked = $bindable(false),
		label,
		description,
		disabled = false,
		id,
		name,
		...rest
	}: Props = $props();
	const reactiveId = $derived(id ?? `sw-${Math.random().toString(36).slice(2, 9)}`);
</script>

<label class="switch" class:is-disabled={disabled} for={reactiveId}>
	<input id={reactiveId} {name} type="checkbox" bind:checked {disabled} {...rest} />
	<span class="track" aria-hidden="true">
		<span class="thumb"></span>
	</span>
	{#if label || description}
		<span class="text">
			{#if label}<span class="label">{label}</span>{/if}
			{#if description}<span class="desc">{description}</span>{/if}
		</span>
	{/if}
</label>

<style>
	.switch {
		display: inline-flex;
		align-items: flex-start;
		gap: var(--space-3);
		cursor: pointer;
		user-select: none;
	}
	.switch.is-disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}
	.track {
		flex-shrink: 0;
		width: 44px;
		height: 24px;
		background: var(--surface-3);
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-full);
		position: relative;
		transition: all var(--dur-base) var(--ease-out);
	}
	.thumb {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 18px;
		height: 18px;
		background: var(--ink-200);
		border-radius: var(--radius-full);
		box-shadow: var(--shadow-elev-1);
		transition: all var(--dur-base) var(--ease-spring);
	}
	input:checked + .track {
		background: var(--gradient-gold);
		border-color: transparent;
	}
	input:checked + .track .thumb {
		left: 22px;
		background: var(--surface-0);
	}
	input:focus-visible + .track {
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.25);
	}
	.text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.label {
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		color: var(--ink-100);
	}
	.desc {
		font-size: var(--text-xs);
		color: var(--ink-400);
		line-height: var(--leading-snug);
	}
</style>
