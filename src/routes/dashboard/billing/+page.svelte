<script lang="ts">
	import {
		IconCreditCard,
		IconPlus,
		IconTrash,
		IconStar,
		IconStarFilled,
		IconDownload,
		IconBrandVisa,
		IconBrandMastercard,
		IconReceipt2,
		IconBuilding
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { toasts } from '$lib/stores/toast.svelte.js';

	type Card = { id: string; brand: 'visa' | 'mc'; last4: string; exp: string; default: boolean };

	let cards = $state<Card[]>([
		{ id: 'c1', brand: 'visa', last4: '4242', exp: '04 / 28', default: true },
		{ id: 'c2', brand: 'mc', last4: '8810', exp: '11 / 27', default: false }
	]);

	let addCardOpen = $state(false);
	let removeId = $state<string | null>(null);
	let newCard = $state({ number: '', name: '', exp: '', cvc: '' });

	function setDefault(id: string) {
		cards = cards.map((c) => ({ ...c, default: c.id === id }));
		toasts.success('Default payment method updated.');
	}
	function confirmRemove() {
		if (!removeId) return;
		cards = cards.filter((c) => c.id !== removeId);
		removeId = null;
		toasts.success('Payment method removed.');
	}
	function addCard() {
		if (!newCard.number || !newCard.exp || !newCard.cvc || !newCard.name) {
			toasts.error('All card fields are required.');
			return;
		}
		const id = 'c' + Math.random().toString(36).slice(2, 7);
		const last4 = newCard.number.replace(/\D/g, '').slice(-4) || '0000';
		cards = [...cards, { id, brand: 'visa', last4, exp: newCard.exp, default: cards.length === 0 }];
		addCardOpen = false;
		newCard = { number: '', name: '', exp: '', cvc: '' };
		toasts.success(`Card ending in ${last4} added.`);
	}

	const invoices = [
		{
			id: 'INV-2026-0184',
			date: 'Mar 24, 2026',
			desc: 'Day Trading · Quarterly',
			amount: 697.0,
			status: 'Paid' as const
		},
		{
			id: 'INV-2026-0117',
			date: 'Dec 24, 2025',
			desc: 'Day Trading · Quarterly',
			amount: 697.0,
			status: 'Paid' as const
		},
		{
			id: 'INV-2025-0986',
			date: 'Nov 18, 2025',
			desc: 'Revolution Ranger',
			amount: 997.0,
			status: 'Paid' as const
		},
		{
			id: 'INV-2025-0985',
			date: 'Nov 18, 2025',
			desc: 'Day Trading · Quarterly',
			amount: 697.0,
			status: 'Paid' as const
		}
	];
</script>

<Seo title="Billing" noindex />

<section class="layout">
	<!-- Payment methods -->
	<article class="card">
		<header class="card-h">
			<h3><IconCreditCard size={18} />Payment methods</h3>
			<Button variant="primary" size="sm" onclick={() => (addCardOpen = true)}>
				{#snippet iconLeft()}<IconPlus size={14} />{/snippet}
				Add card
			</Button>
		</header>
		<ul class="cards">
			{#each cards as c (c.id)}
				<li class="card-row" class:is-default={c.default}>
					<div class="brand">
						{#if c.brand === 'visa'}<IconBrandVisa size={28} />{:else}<IconBrandMastercard
								size={28}
							/>{/if}
					</div>
					<div class="card-info">
						<p class="cn">•••• •••• •••• {c.last4}</p>
						<p class="ce">Expires {c.exp}</p>
					</div>
					<div class="card-actions">
						{#if c.default}
							<Badge variant="gold" size="sm"
								>{#snippet children()}<IconStarFilled size={10} />Default{/snippet}</Badge
							>
						{:else}
							<button class="link" type="button" onclick={() => setDefault(c.id)}
								>Set default</button
							>
						{/if}
						<button
							class="icon-btn"
							type="button"
							aria-label="Remove card"
							onclick={() => (removeId = c.id)}
						>
							<IconTrash size={14} />
						</button>
					</div>
				</li>
			{/each}
		</ul>
	</article>

	<!-- Billing address -->
	<article class="card">
		<header class="card-h">
			<h3><IconBuilding size={18} />Billing address</h3>
			<Button variant="gold-outline" size="sm">Edit</Button>
		</header>
		<dl class="addr">
			<div>
				<dt>Name</dt>
				<dd>Alex Morgan</dd>
			</div>
			<div>
				<dt>Company</dt>
				<dd>—</dd>
			</div>
			<div>
				<dt>Address</dt>
				<dd>1234 Lakeside Ave<br />Suite 600<br />Chicago, IL 60601<br />United States</dd>
			</div>
			<div>
				<dt>Tax ID (VAT/GST)</dt>
				<dd>—</dd>
			</div>
			<div>
				<dt>Billing email</dt>
				<dd>alex.morgan@example.com</dd>
			</div>
		</dl>
	</article>

	<!-- Invoices -->
	<article class="card full">
		<header class="card-h">
			<h3><IconReceipt2 size={18} />Invoices</h3>
			<span class="muted">{invoices.length} on file</span>
		</header>
		<div class="table-scroll">
			<table>
				<thead>
					<tr>
						<th>Invoice</th><th>Date</th><th>Description</th><th class="right">Amount</th><th
							>Status</th
						><th></th>
					</tr>
				</thead>
				<tbody>
					{#each invoices as inv}
						<tr>
							<td class="mono">{inv.id}</td>
							<td>{inv.date}</td>
							<td>{inv.desc}</td>
							<td class="right amount">${inv.amount.toFixed(2)}</td>
							<td><Badge variant="success" size="sm">{inv.status}</Badge></td>
							<td class="right">
								<button class="icon-btn" type="button" aria-label="Download invoice">
									<IconDownload size={14} />
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</article>
</section>

<!-- ADD CARD MODAL -->
<Modal
	bind:open={addCardOpen}
	title="Add a payment method"
	description="Card details are tokenized — we never store raw card numbers."
	size="sm"
>
	<div class="form">
		<Input
			label="Cardholder name"
			name="name"
			bind:value={newCard.name}
			placeholder="Alex Morgan"
			autocomplete="cc-name"
		/>
		<Input
			label="Card number"
			name="number"
			bind:value={newCard.number}
			placeholder="1234 5678 9012 3456"
			inputmode="numeric"
			autocomplete="cc-number"
		/>
		<div class="row-2">
			<Input
				label="Expiration"
				name="exp"
				bind:value={newCard.exp}
				placeholder="MM / YY"
				autocomplete="cc-exp"
			/>
			<Input
				label="CVC"
				name="cvc"
				bind:value={newCard.cvc}
				placeholder="123"
				inputmode="numeric"
				autocomplete="cc-csc"
			/>
		</div>
	</div>
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (addCardOpen = false)}>Cancel</Button>
		<Button variant="primary" onclick={addCard}>Save card</Button>
	{/snippet}
</Modal>

<!-- REMOVE CARD CONFIRM -->
<Modal
	open={!!removeId}
	title="Remove payment method?"
	description="You can add it again at any time."
	size="sm"
	onclose={() => (removeId = null)}
>
	<p>
		This card will be removed from your account. If it is your default, you will need to set a new
		default before your next renewal.
	</p>
	{#snippet footer()}
		<Button variant="ghost" onclick={() => (removeId = null)}>Cancel</Button>
		<Button variant="danger" onclick={confirmRemove}>Remove</Button>
	{/snippet}
</Modal>

<style>
	.layout {
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--space-5);
	}
	@media (min-width: 1280px) {
		.layout {
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
	.muted {
		font-size: var(--text-xs);
		color: var(--ink-400);
	}

	.cards {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.card-row {
		display: grid;
		grid-template-columns: 60px 1fr auto;
		gap: var(--space-4);
		align-items: center;
		padding: var(--space-4);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
	}
	.card-row.is-default {
		border-color: var(--border-gold);
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.06), var(--surface-2));
	}
	.brand {
		color: var(--ink-200);
	}
	.cn {
		font-family: var(--font-mono);
		font-size: var(--text-sm);
		color: var(--ink-100);
		margin: 0;
		letter-spacing: 0.05em;
	}
	.ce {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 4px 0 0;
	}
	.card-actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-shrink: 0;
	}
	.link {
		background: transparent;
		color: var(--gold-300);
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		text-decoration: underline;
		border: 0;
	}
	.link:hover {
		color: var(--gold-200);
	}
	.icon-btn {
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-300);
	}
	.icon-btn:hover {
		color: var(--gold-300);
		border-color: var(--border-gold);
	}

	.addr {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.addr > div {
		display: grid;
		grid-template-columns: 140px 1fr;
		gap: var(--space-4);
	}
	.addr dt {
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-wider);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
	}
	.addr dd {
		margin: 0;
		font-size: var(--text-sm);
		color: var(--ink-100);
		line-height: var(--leading-relaxed);
	}

	.table-scroll {
		overflow-x: auto;
	}
	table {
		width: 100%;
		border-collapse: collapse;
	}
	th {
		text-align: left;
		padding: 0 var(--space-4) var(--space-3);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		border-bottom: 1px solid var(--border-default);
	}
	th.right {
		text-align: right;
	}
	td {
		padding: var(--space-4);
		font-size: var(--text-sm);
		color: var(--ink-200);
		border-bottom: 1px solid var(--border-subtle);
	}
	td.right {
		text-align: right;
	}
	td.mono {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
	td.amount {
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
	}

	.form {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.row-2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-4);
	}
</style>
