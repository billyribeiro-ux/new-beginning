<script lang="ts">
	import {
		IconWorld,
		IconSearch,
		IconMailFast,
		IconPlug,
		IconDeviceFloppy
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Textarea from '$lib/components/ui/Textarea.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import Switch from '$lib/components/ui/Switch.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { toasts } from '$lib/stores/toast.svelte.js';

	let tab = $state('general');
	const tabs = [
		{ id: 'general', label: 'General' },
		{ id: 'seo', label: 'SEO' },
		{ id: 'email', label: 'Email' },
		{ id: 'integrations', label: 'Integrations' }
	];

	// General
	let siteName = $state('TradeFlex Trading');
	let supportEmail = $state('hello@tradeflextrading.com');
	let timezone = $state('America/Chicago');
	let currency = $state('USD');

	// SEO
	let defaultDescription = $state(
		'Engineered indicators, education, and a live day-trading desk for serious operators. Built by traders who run a real process.'
	);
	let twitterHandle = $state('@tradeflextrading');
	let ogImage = $state('/og/default.png');
	let indexable = $state(true);

	// Email
	let fromName = $state('TradeFlex Trading');
	let fromEmail = $state('hello@tradeflextrading.com');
	let replyTo = $state('hello@tradeflextrading.com');
	let transactionalProvider = $state('none');

	// Integrations
	const integrations = [
		{
			id: 'stripe',
			name: 'Stripe',
			desc: 'Payment processing and subscriptions.',
			status: 'not connected',
			color: 'var(--info)'
		},
		{
			id: 'mailchimp',
			name: 'ConvertKit',
			desc: 'Email broadcast and newsletter automation.',
			status: 'not connected',
			color: 'var(--success)'
		},
		{
			id: 'plausible',
			name: 'Plausible Analytics',
			desc: 'Privacy-respecting visitor analytics.',
			status: 'connected',
			color: 'var(--gold-400)'
		},
		{
			id: 'discord',
			name: 'Discord',
			desc: 'Member community access provisioning.',
			status: 'connected',
			color: 'var(--info)'
		},
		{
			id: 'sentry',
			name: 'Sentry',
			desc: 'Error monitoring and performance traces.',
			status: 'not connected',
			color: 'var(--danger)'
		}
	];

	function save() {
		toasts.success('Settings saved', 'Phase 1 stub — wires up to backend in a later phase.');
	}
</script>

<Seo title="Admin · Settings" noindex />

<header class="ph">
	<div>
		<p class="eyebrow">System</p>
		<h2>Site settings</h2>
		<p class="muted">Configuration for the public site, SEO defaults, email, and integrations.</p>
	</div>
	<Button variant="primary" onclick={save}>
		{#snippet iconLeft()}<IconDeviceFloppy size={14} />{/snippet}
		Save changes
	</Button>
</header>

<Tabs {tabs} bind:value={tab} />

<div class="panels">
	{#if tab === 'general'}
		<article class="card">
			<h3><IconWorld size={16} />General</h3>
			<div class="grid-2">
				<Input label="Site name" name="site-name" bind:value={siteName} />
				<Input label="Support email" name="support" type="email" bind:value={supportEmail} />
				<Input label="Time zone" name="tz" bind:value={timezone} />
				<Select
					label="Currency"
					name="cur"
					bind:value={currency}
					options={[
						{ value: 'USD', label: 'USD · US Dollar' },
						{ value: 'EUR', label: 'EUR · Euro' },
						{ value: 'GBP', label: 'GBP · Pound Sterling' }
					]}
				/>
			</div>
		</article>
	{:else if tab === 'seo'}
		<article class="card">
			<h3><IconSearch size={16} />SEO defaults</h3>
			<Textarea
				label="Default meta description"
				name="desc"
				bind:value={defaultDescription}
				rows={4}
			/>
			<div class="grid-2">
				<Input label="Twitter / X handle" name="x" bind:value={twitterHandle} />
				<Input label="Default OG image" name="og" bind:value={ogImage} />
			</div>
			<Switch
				bind:checked={indexable}
				label="Allow search engines to index the site"
				description="When off, every page emits noindex,nofollow. Use this for staging environments."
			/>
		</article>
	{:else if tab === 'email'}
		<article class="card">
			<h3><IconMailFast size={16} />Transactional email</h3>
			<div class="grid-2">
				<Input label="From name" name="from-name" bind:value={fromName} />
				<Input label="From email" name="from-email" type="email" bind:value={fromEmail} />
				<Input label="Reply-to" name="reply" type="email" bind:value={replyTo} />
				<Select
					label="Provider"
					name="provider"
					bind:value={transactionalProvider}
					options={[
						{ value: 'none', label: 'None (Phase 1)' },
						{ value: 'resend', label: 'Resend' },
						{ value: 'postmark', label: 'Postmark' },
						{ value: 'ses', label: 'AWS SES' }
					]}
				/>
			</div>
			<p class="note">
				Real ESP wiring is a Phase 2 task. Phase 1 captures still persist via Drizzle.
			</p>
		</article>
	{:else if tab === 'integrations'}
		<article class="card">
			<h3><IconPlug size={16} />Integrations</h3>
			<ul class="ints">
				{#each integrations as i (i.id)}
					<li>
						<span class="i-mark" style:background={i.color}></span>
						<div class="i-body">
							<p class="i-name">{i.name}</p>
							<p class="i-desc">{i.desc}</p>
						</div>
						<div class="i-actions">
							<Badge variant={i.status === 'connected' ? 'success' : 'outline'} size="sm"
								>{i.status}</Badge
							>
							<Button variant={i.status === 'connected' ? 'ghost' : 'gold-outline'} size="sm">
								{i.status === 'connected' ? 'Configure' : 'Connect'}
							</Button>
						</div>
					</li>
				{/each}
			</ul>
		</article>
	{/if}
</div>

<style>
	.ph {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: var(--space-4);
		flex-wrap: wrap;
		margin-bottom: clamp(2rem, 4vw, 3rem);
	}
	.eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
		margin: 0;
	}
	.eyebrow::before {
		display: none;
	}
	.ph h2 {
		font-family: var(--font-display);
		font-size: clamp(1.75rem, 3vw, 2.5rem);
		margin: var(--space-2) 0;
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
		margin: 0;
	}

	.panels {
		margin-top: var(--space-5);
	}
	.card {
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.card h3 {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-family: var(--font-display);
		font-size: var(--text-lg);
		margin: 0;
		padding-bottom: var(--space-3);
		border-bottom: 1px solid var(--border-default);
	}
	.card h3 :global(svg) {
		color: var(--gold-400);
	}
	.grid-2 {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-4);
	}
	@media (--bp-md) {
		.grid-2 {
			grid-template-columns: 1fr 1fr;
		}
	}
	.note {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
	}

	.ints {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.ints li {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: var(--space-4);
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-default);
	}
	.i-mark {
		width: 4px;
		height: 40px;
		border-radius: var(--radius-full);
		flex-shrink: 0;
	}
	.i-name {
		font-size: var(--text-sm);
		color: var(--ink-100);
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.i-desc {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
	.i-actions {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		flex-wrap: wrap;
	}
</style>
