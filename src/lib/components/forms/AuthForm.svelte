<script lang="ts">
	import {
		IconBrandGoogleFilled,
		IconBrandApple,
		IconBrandGithub,
		IconArrowRight
	} from '@tabler/icons-svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import PasswordField from './PasswordField.svelte';
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import type { SubmitFunction } from '@sveltejs/kit';

	type Mode = 'login' | 'signup' | 'forgot' | 'reset';
	type Props = {
		mode: Mode;
		action?: string;
		title?: string;
		subtitle?: string;
	};
	let { mode, action = '', title, subtitle }: Props = $props();

	let email = $state('');
	let password = $state('');
	let confirm = $state('');
	let name = $state('');
	let remember = $state(true);
	let terms = $state(false);
	let submitting = $state(false);
	let formError = $state('');

	const labels = $derived(
		{
			login: {
				title: title ?? 'Welcome back',
				subtitle: subtitle ?? 'Sign in to access your desk.',
				cta: 'Sign in'
			},
			signup: {
				title: title ?? 'Create your account',
				subtitle: subtitle ?? 'Start with a 14-day refund window.',
				cta: 'Create account'
			},
			forgot: {
				title: title ?? 'Forgot password',
				subtitle: subtitle ?? 'We will send you a reset link.',
				cta: 'Send reset link'
			},
			reset: {
				title: title ?? 'Set a new password',
				subtitle: subtitle ?? 'Choose something strong and memorable.',
				cta: 'Update password'
			}
		}[mode]
	);

	const submit: SubmitFunction = () => {
		submitting = true;
		formError = '';
		return async ({ result, update }) => {
			submitting = false;
			if (result.type === 'failure') {
				formError =
					(result.data as { error?: string } | undefined)?.error ??
					'Please review the form and try again.';
			}
			await update();
		};
	};
</script>

<div class="auth-card">
	<header class="head">
		<h1>{labels.title}</h1>
		<p>{labels.subtitle}</p>
	</header>

	{#if mode === 'login' || mode === 'signup'}
		<div class="oauth">
			<button class="oauth-btn" type="button" disabled>
				<IconBrandGoogleFilled size={16} />
				Continue with Google
			</button>
			<div class="oauth-row">
				<button class="oauth-btn alt" type="button" disabled aria-label="Continue with Apple">
					<IconBrandApple size={16} />
				</button>
				<button class="oauth-btn alt" type="button" disabled aria-label="Continue with GitHub">
					<IconBrandGithub size={16} />
				</button>
			</div>
			<p class="oauth-note">Social sign-in available soon.</p>
		</div>

		<div class="divider">
			<span>or with email</span>
		</div>
	{/if}

	<form method="POST" {action} use:enhance={submit} novalidate class="form">
		{#if mode === 'signup'}
			<Input
				label="Full name"
				name="name"
				type="text"
				autocomplete="name"
				placeholder="Alex Morgan"
				bind:value={name}
				required
			/>
		{/if}

		{#if mode !== 'reset'}
			<Input
				label="Email"
				name="email"
				type="email"
				autocomplete="email"
				placeholder="your@email.com"
				bind:value={email}
				required
			/>
		{/if}

		{#if mode === 'login'}
			<PasswordField name="password" bind:value={password} autocomplete="current-password" />
			<div class="row-between">
				<Checkbox label="Remember me" bind:checked={remember} />
				<a class="link" href={resolve('/forgot-password')}>Forgot password?</a>
			</div>
		{/if}

		{#if mode === 'signup'}
			<PasswordField
				name="password"
				bind:value={password}
				strengthMeter
				autocomplete="new-password"
			/>
			<PasswordField
				name="confirm"
				bind:value={confirm}
				autocomplete="new-password"
				label="Confirm password"
			/>
			<Checkbox bind:checked={terms} name="terms">
				I agree to the <a href={resolve('/legal/terms')}>Terms</a> and
				<a href={resolve('/legal/privacy')}>Privacy Policy</a>.
			</Checkbox>
		{/if}

		{#if mode === 'reset'}
			<PasswordField
				name="password"
				bind:value={password}
				strengthMeter
				autocomplete="new-password"
				label="New password"
			/>
			<PasswordField
				name="confirm"
				bind:value={confirm}
				autocomplete="new-password"
				label="Confirm new password"
			/>
		{/if}

		{#if formError}
			<p class="form-err" role="alert">{formError}</p>
		{/if}

		<Button type="submit" variant="primary" size="lg" fullWidth loading={submitting}>
			{labels.cta}
			{#snippet iconRight()}<IconArrowRight size={16} />{/snippet}
		</Button>
	</form>

	<footer class="alt-link">
		{#if mode === 'login'}
			<p>New here? <a href={resolve('/signup')}>Create an account</a></p>
		{:else if mode === 'signup'}
			<p>Already have an account? <a href={resolve('/login')}>Sign in</a></p>
		{:else}
			<p>Remembered it? <a href={resolve('/login')}>Back to sign in</a></p>
		{/if}
	</footer>
</div>

<style>
	.auth-card {
		width: 100%;
		max-width: 460px;
		margin-inline: auto;
		padding: clamp(2rem, 4vw, 3rem);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-2xl);
		box-shadow: var(--shadow-elev-3);
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}
	.head {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	h1 {
		font-family: var(--font-display);
		font-size: var(--text-3xl);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.head p {
		color: var(--ink-300);
		font-size: var(--text-sm);
		margin: 0;
		line-height: var(--leading-relaxed);
	}

	.oauth {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.oauth-btn {
		width: 100%;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		height: 48px;
		font-family: inherit;
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		cursor: not-allowed;
		opacity: 0.85;
		transition: all var(--dur-fast) var(--ease-out);
	}
	.oauth-btn.alt {
		width: auto;
		flex: 1;
	}
	.oauth-row {
		display: flex;
		gap: var(--space-2);
	}
	.oauth-note {
		text-align: center;
		font-size: var(--text-2xs);
		color: var(--ink-400);
		margin: 0;
	}

	.divider {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
	}
	.divider::before,
	.divider::after {
		content: '';
		flex: 1;
		height: 1px;
		background: var(--border-default);
	}
	.divider span {
		padding-inline: var(--space-3);
	}

	.form {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.row-between {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	.link {
		color: var(--gold-300);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
	}
	.link:hover {
		color: var(--gold-200);
		text-decoration: underline;
	}
	.form-err {
		color: var(--danger);
		background: var(--danger-bg);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-md);
		font-size: var(--text-xs);
		margin: 0;
	}

	.alt-link {
		text-align: center;
	}
	.alt-link p {
		font-size: var(--text-sm);
		color: var(--ink-300);
		margin: 0;
	}
	.alt-link a {
		color: var(--gold-300);
		font-weight: var(--weight-semibold);
	}
	.alt-link a:hover {
		color: var(--gold-200);
		text-decoration: underline;
	}
</style>
