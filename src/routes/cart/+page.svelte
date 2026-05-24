<script lang="ts">
	import {
		IconShoppingBagX,
		IconMinus,
		IconPlus,
		IconTrash,
		IconArrowRight,
		IconLock,
		IconReceiptTax,
		IconGift
	} from '@tabler/icons-svelte';
	import Seo from '$lib/components/seo/Seo.svelte';
	import Breadcrumbs from '$lib/components/seo/Breadcrumbs.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { cart } from '$lib/stores/cart.svelte.js';
	import { formatPrice } from '$lib/utils/money.js';
	import { resolve } from '$app/paths';

	let promoCode = $state('');
	let promoFeedback = $state('');

	function applyPromo() {
		promoFeedback = promoCode.trim() ? 'Promo codes activate at checkout — coming in Phase 2.' : '';
	}
</script>

<Seo title="Cart" description="Review your TradeFlex Trading cart." noindex />

<section class="cart-page">
	<div class="container">
		<Breadcrumbs
			items={[
				{ label: 'Home', href: '/' },
				{ label: 'Cart', href: '/cart' }
			]}
		/>
		<header>
			<h1>Your cart</h1>
			<p class="muted">{cart.count} {cart.count === 1 ? 'item' : 'items'} ready for review.</p>
		</header>

		{#if cart.isEmpty}
			<div class="empty">
				<div class="empty-ic"><IconShoppingBagX size={42} /></div>
				<h2>Your cart is empty.</h2>
				<p>Browse the catalog and find your next edge.</p>
				<div class="empty-actions">
					<Button variant="primary" href="/subscription">Subscription tiers</Button>
					<Button variant="gold-outline" href="/indicators">Indicators</Button>
					<Button variant="gold-outline" href="/courses">Courses</Button>
				</div>
			</div>
		{:else}
			<div class="layout">
				<section class="lines-col">
					<table class="lines-table">
						<thead>
							<tr>
								<th colspan="2">Item</th>
								<th class="hide-sm">Qty</th>
								<th class="right">Total</th>
								<th><span class="sr-only">Actions</span></th>
							</tr>
						</thead>
						<tbody>
							{#each cart.lines as line (line.id)}
								<tr>
									<td class="thumb-cell">
										<div
											class="thumb"
											style:background={line.kind === 'plan'
												? 'var(--gradient-gold)'
												: 'linear-gradient(135deg, var(--surface-3), var(--surface-2))'}
											aria-hidden="true"
										>
											<span class="thumb-kind"
												>{line.kind === 'plan' ? 'Subscription' : line.kind}</span
											>
										</div>
									</td>
									<td>
										<p class="line-name">{line.name}</p>
										{#if line.subtitle}<p class="line-sub">{line.subtitle}</p>{/if}
										<div class="qty mobile-qty">
											<button
												type="button"
												aria-label="Decrease"
												onclick={() => cart.setQuantity(line.id, line.quantity - 1)}
												><IconMinus size={12} /></button
											>
											<span>{line.quantity}</span>
											<button
												type="button"
												aria-label="Increase"
												onclick={() => cart.setQuantity(line.id, line.quantity + 1)}
												><IconPlus size={12} /></button
											>
										</div>
									</td>
									<td class="hide-sm">
										<div class="qty">
											<button
												type="button"
												aria-label="Decrease"
												onclick={() => cart.setQuantity(line.id, line.quantity - 1)}
												><IconMinus size={12} /></button
											>
											<span>{line.quantity}</span>
											<button
												type="button"
												aria-label="Increase"
												onclick={() => cart.setQuantity(line.id, line.quantity + 1)}
												><IconPlus size={12} /></button
											>
										</div>
									</td>
									<td class="right total">{formatPrice(line.priceCents * line.quantity)}</td>
									<td>
										<button
											class="remove"
											type="button"
											aria-label="Remove"
											onclick={() => cart.remove(line.id)}
										>
											<IconTrash size={16} />
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>

					<div class="cart-actions">
						<a class="continue" href={resolve('/indicators')}>← Continue shopping</a>
						<button type="button" class="clear" onclick={() => cart.clear()}>Empty cart</button>
					</div>
				</section>

				<aside class="summary-col">
					<div class="summary">
						<h2>Order summary</h2>

						<div class="promo">
							<Input
								label="Promo code"
								name="promo"
								placeholder="e.g. TF2026"
								bind:value={promoCode}
							/>
							<Button variant="outline" size="md" onclick={applyPromo}>Apply</Button>
						</div>
						{#if promoFeedback}<p class="promo-feedback">
								<IconGift size={12} />
								{promoFeedback}
							</p>{/if}

						<dl class="totals">
							<div class="row">
								<dt>Subtotal</dt>
								<dd>{formatPrice(cart.subtotalCents)}</dd>
							</div>
							<div class="row">
								<dt>Estimated tax</dt>
								<dd class="muted">Calculated at checkout</dd>
							</div>
							<div class="row row-total">
								<dt>Total today</dt>
								<dd>{formatPrice(cart.subtotalCents)}</dd>
							</div>
						</dl>

						<Tooltip label="Checkout opens in Phase 2" side="top">
							<Button variant="primary" size="lg" fullWidth disabled>
								{#snippet iconLeft()}<IconLock size={16} />{/snippet}
								Secure checkout
								{#snippet iconRight()}<IconArrowRight size={16} />{/snippet}
							</Button>
						</Tooltip>

						<ul class="trust">
							<li><IconReceiptTax size={14} /> 14-day refund window on all products</li>
							<li><IconLock size={14} /> Encrypted checkout · industry-grade</li>
						</ul>
					</div>
				</aside>
			</div>
		{/if}
	</div>
</section>

<style>
	.cart-page {
		padding: clamp(3rem, 6vw, 5rem) 0;
	}
	header {
		margin: var(--space-6) 0 clamp(2rem, 4vw, 3rem);
	}
	h1 {
		font-family: var(--font-display);
		font-size: clamp(2.25rem, 4vw, 3.5rem);
		margin: 0 0 var(--space-2);
	}
	.muted {
		color: var(--ink-400);
		font-size: var(--text-sm);
	}

	.empty {
		text-align: center;
		padding: clamp(3rem, 8vw, 6rem) var(--space-4);
		background: var(--surface-1);
		border: 1px dashed var(--border-default);
		border-radius: var(--radius-xl);
	}
	.empty-ic {
		width: 80px;
		height: 80px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		color: var(--ink-400);
		border-radius: var(--radius-full);
		margin-bottom: var(--space-3);
	}
	.empty h2 {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		margin: 0;
	}
	.empty p {
		color: var(--ink-300);
		margin: var(--space-2) 0 var(--space-5);
	}
	.empty-actions {
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
		justify-content: center;
	}

	.layout {
		display: grid;
		grid-template-columns: 1fr;
		gap: clamp(2rem, 4vw, 3rem);
	}
	@media (min-width: 1024px) {
		.layout {
			grid-template-columns: 1.6fr 1fr;
		}
	}

	.lines-table {
		width: 100%;
		border-collapse: collapse;
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xl);
		overflow: hidden;
	}
	.lines-table thead {
		background: var(--surface-2);
	}
	.lines-table th {
		padding: var(--space-4);
		font-size: var(--text-xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--ink-400);
		font-weight: var(--weight-semibold);
		text-align: left;
	}
	.lines-table th.right {
		text-align: right;
	}
	.lines-table td {
		padding: var(--space-4);
		border-top: 1px solid var(--border-subtle);
		vertical-align: middle;
	}
	.lines-table .right {
		text-align: right;
	}
	.thumb-cell {
		width: 90px;
	}
	.thumb {
		width: 80px;
		height: 80px;
		border-radius: var(--radius-md);
		display: flex;
		align-items: flex-end;
		padding: var(--space-2);
	}
	.thumb-kind {
		font-size: 9px;
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: rgba(0, 0, 0, 0.7);
		background: rgba(255, 255, 255, 0.75);
		padding: 2px 6px;
		border-radius: var(--radius-xs);
	}
	.line-name {
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		font-size: var(--text-md);
		margin: 0;
	}
	.line-sub {
		color: var(--ink-400);
		font-size: var(--text-xs);
		margin: 4px 0 0;
	}
	.qty {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		padding: 2px;
	}
	.qty button {
		width: 28px;
		height: 28px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		color: var(--ink-300);
	}
	.qty button:hover {
		color: var(--ink-100);
		background: var(--surface-3);
	}
	.qty span {
		min-width: 20px;
		text-align: center;
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
	}
	.mobile-qty {
		margin-top: var(--space-2);
	}
	.total {
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		font-size: var(--text-md);
		color: var(--ink-100);
	}
	.remove {
		color: var(--ink-400);
		padding: var(--space-2);
		border-radius: var(--radius-xs);
	}
	.remove:hover {
		color: var(--danger);
		background: rgba(217, 104, 104, 0.08);
	}

	@media (max-width: 767px) {
		.hide-sm {
			display: none;
		}
	}
	@media (min-width: 768px) {
		.mobile-qty {
			display: none;
		}
	}

	.cart-actions {
		margin-top: var(--space-5);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.continue {
		color: var(--ink-300);
		font-size: var(--text-sm);
		text-decoration: none;
		font-weight: var(--weight-medium);
	}
	.continue:hover {
		color: var(--gold-300);
	}
	.clear {
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-decoration: underline;
		background: transparent;
		border: 0;
	}

	.summary {
		position: sticky;
		top: calc(var(--navbar-height) + var(--space-4));
		padding: var(--space-7);
		background: var(--surface-1);
		border: 1px solid var(--border-gold);
		border-radius: var(--radius-2xl);
		box-shadow: var(--shadow-elev-2), var(--glow-gold);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}
	.summary h2 {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		margin: 0;
	}
	.promo {
		display: flex;
		gap: var(--space-3);
		align-items: flex-end;
	}
	.promo :global(.field) {
		flex: 1;
	}
	.promo-feedback {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-xs);
		color: var(--gold-400);
		margin: 0;
	}

	.totals {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding-block: var(--space-4);
		border-block: 1px solid var(--border-default);
	}
	.row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		font-size: var(--text-sm);
		color: var(--ink-200);
	}
	.row dt {
		color: var(--ink-300);
	}
	.row dd {
		margin: 0;
		font-weight: var(--weight-semibold);
	}
	.row.row-total {
		padding-top: var(--space-3);
		border-top: 1px solid var(--border-default);
	}
	.row.row-total dt,
	.row.row-total dd {
		font-family: var(--font-display);
		font-size: var(--text-xl);
		color: var(--ink-100);
	}

	.trust {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.trust li {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-xs);
		color: var(--ink-400);
	}
	.trust li :global(svg) {
		color: var(--gold-400);
	}
</style>
