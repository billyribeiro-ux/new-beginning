<script lang="ts">
	import { IconSend, IconCheck } from '@tabler/icons-svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Textarea from '$lib/components/ui/Textarea.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { enhance } from '$app/forms';
	import type { SubmitFunction } from '@sveltejs/kit';

	let name = $state('');
	let email = $state('');
	let subject = $state('');
	let body = $state('');
	let website = $state(''); // honeypot
	let submitting = $state(false);
	let success = $state(false);
	let error = $state('');

	const submit: SubmitFunction = () => {
		submitting = true;
		error = '';
		return async ({ result, update }) => {
			submitting = false;
			if (result.type === 'success') {
				success = true;
				name = email = subject = body = '';
				await update({ reset: false });
			} else if (result.type === 'failure') {
				error =
					(result.data as { error?: string } | undefined)?.error ?? 'Please review and try again.';
			} else {
				await update();
			}
		};
	};
</script>

{#if success}
	<div class="success-panel">
		<span class="ic"><IconCheck size={20} stroke={3} /></span>
		<div>
			<h3>Message received.</h3>
			<p>We will get back to you within one business day at the email you provided.</p>
		</div>
	</div>
{:else}
	<form method="POST" use:enhance={submit} novalidate class="form">
		<input
			type="text"
			name="website"
			tabindex="-1"
			autocomplete="off"
			aria-hidden="true"
			value={website}
			oninput={(e) => (website = e.currentTarget.value)}
			class="hp"
		/>

		<div class="row-2">
			<Input
				label="Name"
				name="name"
				bind:value={name}
				required
				autocomplete="name"
				placeholder="Alex Morgan"
			/>
			<Input
				label="Email"
				name="email"
				type="email"
				bind:value={email}
				required
				autocomplete="email"
				placeholder="alex@example.com"
			/>
		</div>
		<Input
			label="Subject"
			name="subject"
			bind:value={subject}
			required
			placeholder="Question about Revolution Ranger"
		/>
		<Textarea
			label="Message"
			name="body"
			bind:value={body}
			required
			rows={6}
			placeholder="Tell us how we can help…"
		/>

		{#if error}<p class="err" role="alert">{error}</p>{/if}

		<Button type="submit" variant="primary" size="lg" loading={submitting}>
			{#snippet iconLeft()}<IconSend size={16} />{/snippet}
			Send message
		</Button>
	</form>
{/if}

<style>
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.row-2 {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
	}
	@media (min-width: 640px) {
		.row-2 {
			grid-template-columns: 1fr 1fr;
		}
	}
	.hp {
		position: absolute;
		left: -9999px;
		width: 1px;
		height: 1px;
		opacity: 0;
	}
	.err {
		color: var(--danger);
		background: var(--danger-bg);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-md);
		font-size: var(--text-xs);
		margin: 0;
	}
	.success-panel {
		display: flex;
		gap: var(--space-4);
		padding: var(--space-6);
		background: var(--success-bg);
		border: 1px solid var(--success);
		border-radius: var(--radius-lg);
	}
	.ic {
		flex-shrink: 0;
		width: 40px;
		height: 40px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--success);
		color: var(--surface-0);
		border-radius: var(--radius-full);
	}
	.success-panel h3 {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		color: var(--ink-100);
		margin: 0;
	}
	.success-panel p {
		font-size: var(--text-sm);
		color: var(--ink-200);
		margin: 4px 0 0;
		line-height: var(--leading-relaxed);
	}
</style>
