<script lang="ts">
	import type { PageData } from './$types';
	import {
		IconUser,
		IconMail,
		IconLockSquareRoundedFilled,
		IconShieldLock,
		IconDevices,
		IconBrandGoogleFilled,
		IconBrandApple,
		IconKey,
		IconDownload,
		IconAlertTriangle,
		IconCheck
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import PasswordField from '$lib/components/forms/PasswordField.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Switch from '$lib/components/ui/Switch.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import { toasts } from '$lib/stores/toast.svelte.js';
	import { untrack } from 'svelte';

	let { data }: { data: PageData } = $props();
	const user = $derived(data.user);

	let name = $state(untrack(() => data.user?.name ?? ''));
	let email = $state(untrack(() => data.user?.email ?? ''));
	let timezone = $state('America/Chicago');
	let language = $state('en-US');

	let oldPw = $state('');
	let newPw = $state('');
	let confirmPw = $state('');

	let twoFAOpen = $state(false);
	let deleteOpen = $state(false);
	let deleteConfirmText = $state('');

	const initials = $derived(
		user?.name
			.split(' ')
			.map((p) => p[0])
			.slice(0, 2)
			.join('')
			.toUpperCase() ?? 'TF'
	);

	let activeTab = $state('account');
	const tabs = [
		{ id: 'account', label: 'Account' },
		{ id: 'security', label: 'Security' },
		{ id: 'connections', label: 'Connections' },
		{ id: 'sessions', label: 'Sessions' },
		{ id: 'danger', label: 'Danger zone' }
	];

	const sessions = [
		{
			id: 's1',
			device: 'MacBook Pro · Chrome',
			location: 'Chicago, IL · USA',
			last: 'Active now',
			current: true
		},
		{
			id: 's2',
			device: 'iPhone 16 · Safari',
			location: 'Chicago, IL · USA',
			last: '14 hours ago',
			current: false
		},
		{
			id: 's3',
			device: 'iPad Pro · Safari',
			location: 'New York, NY · USA',
			last: '3 days ago',
			current: false
		}
	];

	function saveProfile() {
		toasts.success('Profile updated.');
	}
	function changeEmail() {
		toasts.info('Verification email sent', `Confirm at ${email}.`);
	}
	function changePassword() {
		if (!oldPw || !newPw || newPw !== confirmPw) {
			toasts.error('Please review the password fields.');
			return;
		}
		oldPw = newPw = confirmPw = '';
		toasts.success('Password updated.');
	}
	function revokeSession(id: string) {
		toasts.info(`Session ${id} revoked.`);
	}
	function deleteAccount() {
		if (deleteConfirmText !== 'DELETE') {
			toasts.error('Please type DELETE to confirm.');
			return;
		}
		deleteOpen = false;
		toasts.info('Account deletion requested.', 'You will receive a confirmation email.');
	}
</script>

<Seo title="Profile" noindex />

<div class="hero">
	<div class="avatar-row">
		<div class="avatar-lg">{initials}</div>
		<div>
			<h2>{user?.name}</h2>
			<p class="muted">{user?.email} · Member since Nov 2025</p>
			<button type="button" class="link">Upload new photo</button>
		</div>
	</div>
</div>

<Tabs {tabs} bind:value={activeTab} />

{#if activeTab === 'account'}
	<section class="grid">
		<article class="card">
			<header class="card-h">
				<h3>Profile information</h3>
				<Button variant="primary" size="sm" onclick={saveProfile}>Save changes</Button>
			</header>
			<div class="form-grid">
				<Input label="Full name" name="name" bind:value={name} autocomplete="name" />
				<Input label="Headline" name="headline" placeholder="e.g. Day trader · NQ specialist" />
				<Input label="Time zone" name="tz" bind:value={timezone} />
				<Input label="Language" name="lang" bind:value={language} />
			</div>
		</article>

		<article class="card">
			<header class="card-h">
				<h3><IconMail size={16} />Email</h3>
				<Badge variant="success" size="sm">Verified</Badge>
			</header>
			<p class="muted">
				Your current email is <strong>{user?.email}</strong>. Updating sends a verification link to
				the new address.
			</p>
			<div class="form-row">
				<Input label="New email" name="email" type="email" bind:value={email} />
				<Button variant="gold-outline" onclick={changeEmail}>Send verification</Button>
			</div>
		</article>
	</section>
{:else if activeTab === 'security'}
	<section class="grid">
		<article class="card">
			<header class="card-h">
				<h3><IconLockSquareRoundedFilled size={16} />Password</h3>
				<Badge variant="default" size="sm">Last changed · 3 months ago</Badge>
			</header>
			<div class="form-grid">
				<PasswordField
					name="old-pw"
					label="Current password"
					bind:value={oldPw}
					autocomplete="current-password"
				/>
				<PasswordField
					name="new-pw"
					label="New password"
					bind:value={newPw}
					strengthMeter
					autocomplete="new-password"
				/>
				<PasswordField
					name="confirm-pw"
					label="Confirm new password"
					bind:value={confirmPw}
					autocomplete="new-password"
				/>
			</div>
			<div class="card-actions">
				<Button variant="primary" onclick={changePassword}>Update password</Button>
				<p class="hint">We will sign you out of all other sessions when your password changes.</p>
			</div>
		</article>

		<article class="card">
			<header class="card-h">
				<h3><IconShieldLock size={16} />Two-factor authentication</h3>
				<Badge variant="warning" size="sm">Not enabled</Badge>
			</header>
			<p class="muted">Add a TOTP-based 2FA code on top of your password. Strongly recommended.</p>
			<div class="card-actions">
				<Button variant="primary" onclick={() => (twoFAOpen = true)}>
					{#snippet iconLeft()}<IconKey size={14} />{/snippet}
					Enable 2FA
				</Button>
				<Button variant="gold-outline">Backup codes</Button>
			</div>
		</article>
	</section>
{:else if activeTab === 'connections'}
	<section class="grid">
		<article class="card">
			<header class="card-h"><h3>Connected accounts</h3></header>
			<ul class="conn-list">
				<li>
					<span class="conn-ic"><IconBrandGoogleFilled size={18} /></span>
					<div>
						<p class="ct">Google</p>
						<p class="cs">Not connected</p>
					</div>
					<Button variant="gold-outline" size="sm" disabled>Connect</Button>
				</li>
				<li>
					<span class="conn-ic"><IconBrandApple size={18} /></span>
					<div>
						<p class="ct">Apple</p>
						<p class="cs">Not connected</p>
					</div>
					<Button variant="gold-outline" size="sm" disabled>Connect</Button>
				</li>
			</ul>
		</article>
		<article class="card">
			<header class="card-h"><h3><IconDownload size={16} />Export your data</h3></header>
			<p class="muted">
				Download a JSON archive of your profile, purchases, course progress, and account activity.
			</p>
			<div class="card-actions">
				<Button variant="primary">
					{#snippet iconLeft()}<IconDownload size={14} />{/snippet}
					Request export
				</Button>
			</div>
		</article>
	</section>
{:else if activeTab === 'sessions'}
	<article class="card full">
		<header class="card-h">
			<h3><IconDevices size={16} />Active sessions</h3>
			<Button variant="gold-outline" size="sm">Sign out all other sessions</Button>
		</header>
		<ul class="sessions">
			{#each sessions as s (s.id)}
				<li>
					<div>
						<p class="st">{s.device}</p>
						<p class="sm">{s.location} · {s.last}</p>
					</div>
					{#if s.current}
						<Badge variant="success" size="sm"><IconCheck size={10} />This device</Badge>
					{:else}
						<Button variant="ghost" size="sm" onclick={() => revokeSession(s.id)}>Revoke</Button>
					{/if}
				</li>
			{/each}
		</ul>
	</article>
{:else if activeTab === 'danger'}
	<article class="card danger">
		<header class="card-h">
			<h3><IconAlertTriangle size={16} />Delete account</h3>
			<Badge variant="danger" size="sm">Irreversible</Badge>
		</header>
		<p class="muted">
			This permanently removes your profile, learning progress, downloads, and access to all
			purchases. Refunds for active subscriptions are processed separately.
		</p>
		<div class="card-actions">
			<Button variant="danger" onclick={() => (deleteOpen = true)}>Delete my account</Button>
		</div>
	</article>
{/if}

<Modal
	bind:open={twoFAOpen}
	title="Enable two-factor authentication"
	description="Scan the QR with an authenticator app (Authy, 1Password, Google Authenticator)."
	size="md"
>
	<div class="twofa">
		<div class="qr" aria-hidden="true">
			<div class="qr-grid"></div>
		</div>
		<div>
			<p class="qr-label">Manual code</p>
			<p class="qr-code">TFTR-XKZL-MNB9-2QQ7</p>
			<p class="muted">
				Backup codes will be shown once after activation. Store them in a password manager.
			</p>
		</div>
	</div>
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (twoFAOpen = false)}>Cancel</Button>
		<Button
			variant="primary"
			onclick={() => {
				twoFAOpen = false;
				toasts.success('2FA enabled.');
			}}>I scanned the code</Button
		>
	{/snippet}
</Modal>

<Modal
	bind:open={deleteOpen}
	title="Delete account?"
	description="This cannot be undone."
	size="sm"
>
	<p class="muted">
		Type <strong>DELETE</strong> to confirm. We will email you a summary and process the deletion within
		30 days.
	</p>
	<Input label="Type DELETE" name="confirm" bind:value={deleteConfirmText} placeholder="DELETE" />
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (deleteOpen = false)}>Cancel</Button>
		<Button variant="danger" onclick={deleteAccount}>Confirm deletion</Button>
	{/snippet}
</Modal>

<style>
	.hero {
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	.avatar-row {
		display: flex;
		gap: var(--space-5);
		align-items: center;
	}
	.avatar-lg {
		width: 96px;
		height: 96px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		border-radius: var(--radius-full);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-family: var(--font-display);
		font-weight: var(--weight-bold);
		font-size: var(--text-3xl);
		box-shadow: 0 8px 24px rgba(212, 162, 76, 0.32);
	}
	.hero h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: var(--space-2) 0;
	}
	.link {
		background: transparent;
		color: var(--gold-300);
		font-size: var(--text-xs);
		text-decoration: underline;
		padding: 0;
		border: 0;
	}
	.link:hover {
		color: var(--gold-200);
	}

	.grid {
		margin-top: var(--space-5);
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (min-width: 1024px) {
		.grid {
			grid-template-columns: 1fr 1fr;
		}
		.full {
			grid-column: 1 / -1;
		}
	}

	.card {
		padding: var(--space-6);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.card.danger {
		border-color: var(--danger);
		background: linear-gradient(135deg, rgba(217, 104, 104, 0.04), var(--surface-1));
	}
	.card-h {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-5);
		gap: var(--space-3);
		flex-wrap: wrap;
	}
	.card-h h3 {
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin: 0;
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.card-h h3 :global(svg) {
		color: var(--gold-400);
	}
	.danger .card-h h3 :global(svg) {
		color: var(--danger);
	}

	.form-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
	}
	@media (min-width: 640px) {
		.form-grid {
			grid-template-columns: 1fr 1fr;
		}
	}
	.form-row {
		display: flex;
		gap: var(--space-3);
		align-items: flex-end;
		margin-top: var(--space-4);
	}
	.form-row :global(.field) {
		flex: 1;
	}

	.card-actions {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		margin-top: var(--space-5);
		flex-wrap: wrap;
	}
	.hint {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
	}

	.conn-list {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.conn-list li {
		display: grid;
		grid-template-columns: 40px 1fr auto;
		gap: var(--space-4);
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
	}
	.conn-ic {
		width: 40px;
		height: 40px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-3);
		color: var(--ink-200);
		border-radius: var(--radius-full);
	}
	.ct {
		font-size: var(--text-sm);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		margin: 0;
	}
	.cs {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}

	.sessions {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.sessions li {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
		gap: var(--space-3);
	}
	.st {
		font-size: var(--text-sm);
		color: var(--ink-100);
		font-weight: var(--weight-medium);
		margin: 0;
	}
	.sm {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}

	.twofa {
		display: grid;
		grid-template-columns: 160px 1fr;
		gap: var(--space-5);
		align-items: center;
	}
	@media (max-width: 640px) {
		.twofa {
			grid-template-columns: 1fr;
			text-align: center;
		}
	}
	.qr {
		width: 160px;
		height: 160px;
		background: var(--ink-100);
		border-radius: var(--radius-md);
		padding: var(--space-3);
		display: grid;
		place-items: center;
	}
	.qr-grid {
		width: 100%;
		height: 100%;
		background-image:
			linear-gradient(rgba(0, 0, 0, 0.92) 1px, transparent 1px),
			linear-gradient(90deg, rgba(0, 0, 0, 0.92) 1px, transparent 1px);
		background-size: 8px 8px;
		opacity: 0.92;
	}
	.qr-label {
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.qr-code {
		font-family: var(--font-mono);
		font-size: var(--text-md);
		color: var(--gold-300);
		margin: var(--space-2) 0;
		letter-spacing: 0.1em;
	}
</style>
