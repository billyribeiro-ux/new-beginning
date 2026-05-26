<script lang="ts">
	import type { HTMLSelectAttributes } from 'svelte/elements';
	import IconChevronDown from '@tabler/icons-svelte/icons/chevron-down';

	type Option = { value: string; label: string };
	type Props = HTMLSelectAttributes & {
		label?: string;
		value?: string;
		options: Option[];
	};

	let { label, options, value = $bindable(''), id, ...rest }: Props = $props();
	const reactiveId = $derived(id ?? `sel-${Math.random().toString(36).slice(2, 9)}`);
</script>

<div class="field">
	{#if label}<label for={reactiveId}>{label}</label>{/if}
	<div class="wrap">
		<select id={reactiveId} bind:value {...rest}>
			{#each options as opt (opt.value)}
				<option value={opt.value}>{opt.label}</option>
			{/each}
		</select>
		<span class="chevron" aria-hidden="true"><IconChevronDown size={16} /></span>
	</div>
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
	.wrap {
		position: relative;
	}
	select {
		appearance: none;
		width: 100%;
		height: 48px;
		padding: 0 var(--space-10) 0 var(--space-4);
		font-family: inherit;
		font-size: var(--text-base);
		color: var(--ink-100);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		transition: all var(--dur-base) var(--ease-out);
	}
	select:focus {
		outline: none;
		border-color: var(--gold-500);
		background: var(--surface-2);
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.18);
	}
	.chevron {
		position: absolute;
		right: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		color: var(--ink-300);
		pointer-events: none;
	}
	option {
		background: var(--surface-1);
		color: var(--ink-100);
	}
</style>
