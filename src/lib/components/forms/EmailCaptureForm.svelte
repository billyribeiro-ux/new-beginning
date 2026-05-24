<script lang="ts">
	import { IconMail, IconArrowRight, IconCheck } from '@tabler/icons-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { enhance } from '$app/forms';
	import type { SubmitFunction } from '@sveltejs/kit';
	import { emailSchema } from '$lib/utils/validators.js';

	type Props = {
		action?: string;
		source?: string;
		successMessage?: string;
		ctaLabel?: string;
		variant?: 'stacked' | 'inline';
	};
	let {
		action = '/free-guide?/subscribe',
		source = 'newsletter',
		successMessage = 'You are in. Check your inbox.',
		ctaLabel = 'Subscribe',
		variant = 'inline'
	}: Props = $props();

	let email = $state('');
	let website = $state(''); // honeypot
	let error = $state('');
	let success = $state(false);
	let submitting = $state(false);

	const submit: SubmitFunction = ({ formData, cancel }) => {
		error = '';
		const parsed = emailSchema.safeParse(formData.get('email'));
		if (!parsed.success) {
			error = parsed.error.issues[0]?.message ?? 'Invalid email';
			cancel();
			return;
		}
		submitting = true;
		return async ({ result, update }) => {
			submitting = false;
			if (result.type === 'success') {
				success = true;
				email = '';
				await update({ reset: false });
			} else if (result.type === 'failure') {
				error =
					(result.data as { error?: string } | undefined)?.error ??
					'Something went wrong. Try again.';
			} else {
				await update();
			}
		};
	};
</script>

{#if success}
	<div class="success">
		<span class="success-icon"><IconCheck size={18} stroke={3} /></span>
		<p>{successMessage}</p>
	</div>
{:else}
	<form method="POST" {action} use:enhance={submit} class="form variant-{variant}" novalidate>
		<input type="hidden" name="source" value={source} />
		<input
			type="text"
			name="website"
			value={website}
			oninput={(e) => (website = e.currentTarget.value)}
			class="hp"
			tabindex="-1"
			autocomplete="off"
			aria-hidden="true"
		/>
		<div class="input-row">
			<span class="leading-icon" aria-hidden="true"><IconMail size={16} /></span>
			<input
				type="email"
				name="email"
				placeholder="your@email.com"
				autocomplete="email"
				required
				bind:value={email}
				aria-invalid={!!error}
				aria-label="Email address"
				disabled={submitting}
			/>
		</div>
		<Button type="submit" variant="primary" size="lg" loading={submitting}>
			{ctaLabel}
			{#snippet iconRight()}<IconArrowRight size={16} />{/snippet}
		</Button>
	</form>
	{#if error}<p class="err" role="alert">{error}</p>{/if}
{/if}

<style>
	.form {
		display: flex;
		gap: var(--space-3);
		width: 100%;
	}
	.variant-stacked {
		flex-direction: column;
	}
	.variant-inline {
		flex-direction: column;
	}
	@media (min-width: 640px) {
		.variant-inline {
			flex-direction: row;
			align-items: stretch;
		}
	}
	.hp {
		position: absolute;
		left: -9999px;
		width: 1px;
		height: 1px;
		opacity: 0;
	}
	.input-row {
		position: relative;
		flex: 1;
	}
	.leading-icon {
		position: absolute;
		left: var(--space-4);
		top: 50%;
		transform: translateY(-50%);
		color: var(--ink-400);
		pointer-events: none;
	}
	input[type='email'] {
		width: 100%;
		height: 52px;
		padding: 0 var(--space-4) 0 var(--space-10);
		font-size: var(--text-base);
		color: var(--ink-100);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		transition: all var(--dur-base) var(--ease-out);
		font-family: inherit;
	}
	input[type='email']::placeholder {
		color: var(--ink-400);
	}
	input[type='email']:focus {
		outline: none;
		border-color: var(--gold-500);
		background: var(--surface-2);
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.18);
	}
	input[type='email'][aria-invalid='true'] {
		border-color: var(--danger);
	}
	.err {
		color: var(--danger);
		font-size: var(--text-xs);
		margin-top: var(--space-2);
	}
	.success {
		display: inline-flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-5);
		background: var(--success-bg);
		color: var(--success);
		border: 1px solid var(--success);
		border-radius: var(--radius-full);
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
	}
	.success-icon {
		display: inline-flex;
		width: 24px;
		height: 24px;
		align-items: center;
		justify-content: center;
		background: var(--success);
		color: var(--surface-0);
		border-radius: var(--radius-full);
	}
	.success p {
		margin: 0;
		color: inherit;
	}
</style>
