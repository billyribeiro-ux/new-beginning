<script lang="ts">
	import { IconEye, IconEyeOff, IconLockSquareRoundedFilled } from '@tabler/icons-svelte';
	import { scorePassword } from '$lib/utils/validators.js';

	type Props = {
		label?: string;
		name: string;
		placeholder?: string;
		value?: string;
		strengthMeter?: boolean;
		autocomplete?: 'current-password' | 'new-password' | 'off';
		error?: string;
	};
	let {
		label = 'Password',
		name,
		placeholder = '••••••••',
		value = $bindable(''),
		strengthMeter = false,
		autocomplete = 'current-password',
		error
	}: Props = $props();

	let visible = $state(false);
	const score = $derived(strengthMeter ? scorePassword(value) : null);
</script>

<div class="field" class:has-error={!!error}>
	<label for={name}>{label}</label>
	<div class="wrap">
		<span class="leading"><IconLockSquareRoundedFilled size={16} /></span>
		<input
			id={name}
			{name}
			type={visible ? 'text' : 'password'}
			{placeholder}
			{autocomplete}
			bind:value
			aria-invalid={!!error}
		/>
		<button
			type="button"
			class="toggle"
			aria-label={visible ? 'Hide password' : 'Show password'}
			aria-pressed={visible}
			onclick={() => (visible = !visible)}
		>
			{#if visible}<IconEyeOff size={16} />{:else}<IconEye size={16} />{/if}
		</button>
	</div>
	{#if score && value.length > 0}
		<div class="meter" aria-label="Password strength: {score.label}">
			<div class="meter-bars" data-score={score.score}>
				{#each Array(4) as _, i}
					<span class="bar" class:active={i < score.score}></span>
				{/each}
			</div>
			<span class="meter-label">{score.label}</span>
		</div>
	{/if}
	{#if error}<p class="err" role="alert">{error}</p>{/if}
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
	.leading {
		position: absolute;
		left: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		color: var(--ink-400);
		pointer-events: none;
	}
	input {
		width: 100%;
		height: 48px;
		padding: 0 var(--space-10) 0 var(--space-9);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		color: var(--ink-100);
		font-family: inherit;
		font-size: var(--text-base);
		transition: all var(--dur-base) var(--ease-out);
	}
	input:focus {
		outline: none;
		border-color: var(--gold-500);
		background: var(--surface-2);
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.18);
	}
	.has-error input {
		border-color: var(--danger);
	}
	.toggle {
		position: absolute;
		right: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		color: var(--ink-400);
		padding: var(--space-1);
		display: inline-flex;
		align-items: center;
	}
	.toggle:hover {
		color: var(--ink-200);
	}
	.meter {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		margin-top: var(--space-1);
	}
	.meter-bars {
		display: inline-flex;
		gap: 4px;
		flex: 1;
	}
	.bar {
		flex: 1;
		height: 4px;
		background: var(--surface-3);
		border-radius: var(--radius-full);
		transition: background var(--dur-base) var(--ease-out);
	}
	.bar.active {
		background: var(--gold-400);
	}
	.meter-bars[data-score='4'] .bar.active {
		background: var(--success);
	}
	.meter-bars[data-score='1'] .bar.active,
	.meter-bars[data-score='0'] .bar.active {
		background: var(--danger);
	}
	.meter-label {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.err {
		color: var(--danger);
		font-size: var(--text-xs);
	}
</style>
