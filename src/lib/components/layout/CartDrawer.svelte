<script lang="ts">
	import {
		IconX,
		IconMinus,
		IconPlus,
		IconTrash,
		IconShoppingBagX,
		IconArrowRight,
		IconLock
	} from '@tabler/icons-svelte';
	import { cart } from '$lib/stores/cart.svelte.js';
	import { formatPrice } from '$lib/utils/money.js';
	import Button from '$lib/components/ui/Button.svelte';
	import Tooltip from '$lib/components/ui/Tooltip.svelte';
	import { fly, fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { focusTrap } from '$lib/utils/focusTrap.js';

	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape' && cart.isOpen) cart.close();
	}

	$effect(() => {
		if (cart.isOpen) {
			document.body.style.overflow = 'hidden';
		} else {
			document.body.style.overflow = '';
		}
		return () => {
			document.body.style.overflow = '';
		};
	});
</script>

<svelte:window onkeydown={onKey} />

{#if cart.isOpen}
	<div class="cart-overlay">
		<button
			class="backdrop"
			type="button"
			aria-label="Close cart"
			onclick={() => cart.close()}
			transition:fade={{ duration: 180 }}
		></button>
		<div
			class="drawer"
			role="dialog"
			aria-modal="true"
			aria-label="Shopping cart"
			tabindex="-1"
			transition:fly={{ x: 480, duration: 320, easing: cubicOut }}
			{@attach focusTrap}
		>
			<header class="drawer-header">
				<div>
					<p class="eyebrow">Your cart</p>
					<h3>{cart.count} {cart.count === 1 ? 'item' : 'items'}</h3>
				</div>
				<button class="close" type="button" aria-label="Close cart" onclick={() => cart.close()}>
					<IconX size={18} />
				</button>
			</header>

			<div class="drawer-body">
				{#if cart.isEmpty}
					<div class="empty">
						<div class="empty-icon"><IconShoppingBagX size={42} /></div>
						<h4>Your cart is empty</h4>
						<p>Add an indicator, course, or subscription to get started.</p>
						<Button variant="primary" href="/subscription" size="md">Explore subscriptions</Button>
					</div>
				{:else}
					<ul class="lines">
						{#each cart.lines as line (line.id)}
							<li class="line">
								<div
									class="line-thumb"
									style:background={line.kind === 'plan'
										? 'var(--gradient-gold)'
										: 'linear-gradient(135deg, var(--surface-3), var(--surface-2))'}
								>
									<span class="line-kind"
										>{line.kind === 'plan' ? 'Subscription' : (line.subtitle ?? line.kind)}</span
									>
								</div>
								<div class="line-body">
									<p class="line-name">{line.name}</p>
									{#if line.subtitle}<p class="line-sub">{line.subtitle}</p>{/if}
									<div class="line-row">
										<div class="qty">
											<button
												type="button"
												aria-label="Decrease quantity"
												onclick={() => cart.setQuantity(line.id, line.quantity - 1)}
											>
												<IconMinus size={12} />
											</button>
											<span>{line.quantity}</span>
											<button
												type="button"
												aria-label="Increase quantity"
												onclick={() => cart.setQuantity(line.id, line.quantity + 1)}
											>
												<IconPlus size={12} />
											</button>
										</div>
										<button
											class="remove"
											type="button"
											aria-label="Remove from cart"
											onclick={() => cart.remove(line.id)}
										>
											<IconTrash size={14} />
										</button>
									</div>
								</div>
								<div class="line-price">{formatPrice(line.priceCents * line.quantity)}</div>
							</li>
						{/each}
					</ul>
				{/if}
			</div>

			{#if !cart.isEmpty}
				<footer class="drawer-footer">
					<div class="totals">
						<div class="row-between">
							<span>Subtotal</span>
							<strong>{formatPrice(cart.subtotalCents)}</strong>
						</div>
						<p class="taxes-note">Taxes and any applicable discounts are calculated at checkout.</p>
					</div>
					<Tooltip label="Checkout coming in Phase 2" side="top">
						<Button variant="primary" size="lg" fullWidth disabled>
							{#snippet iconLeft()}<IconLock size={16} />{/snippet}
							Continue to checkout
							{#snippet iconRight()}<IconArrowRight size={16} />{/snippet}
						</Button>
					</Tooltip>
					<button class="clear-btn" type="button" onclick={() => cart.clear()}>Empty cart</button>
				</footer>
			{/if}
		</div>
	</div>
{/if}

<style>
	.cart-overlay {
		position: fixed;
		inset: 0;
		z-index: var(--z-modal);
	}
	.backdrop {
		position: absolute;
		inset: 0;
		background: var(--surface-overlay);
		backdrop-filter: blur(8px);
		-webkit-backdrop-filter: blur(8px);
		border: 0;
	}
	.drawer {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: 100%;
		max-width: 460px;
		background: var(--surface-1);
		border-left: 1px solid var(--border-default);
		display: grid;
		grid-template-rows: auto 1fr auto;
		box-shadow: var(--shadow-elev-4);
	}
	.drawer-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-6);
		border-bottom: 1px solid var(--border-default);
	}
	.drawer-header h3 {
		font-size: var(--text-xl);
		margin: 4px 0 0;
	}
	.eyebrow {
		font-size: var(--text-2xs);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
	}
	.eyebrow::before {
		display: none;
	}
	.close {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-full);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		color: var(--ink-200);
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}
	.close:hover {
		color: var(--ink-100);
		background: var(--surface-3);
	}

	.drawer-body {
		padding: var(--space-4) var(--space-6);
		overflow-y: auto;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: var(--space-3);
		padding: var(--space-12) var(--space-4);
	}
	.empty-icon {
		width: 80px;
		height: 80px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border-radius: var(--radius-full);
		color: var(--ink-400);
		margin-bottom: var(--space-3);
	}
	.empty h4 {
		font-size: var(--text-lg);
	}
	.empty p {
		color: var(--ink-300);
		font-size: var(--text-sm);
		margin-bottom: var(--space-3);
	}

	.lines {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.line {
		display: grid;
		grid-template-columns: 76px 1fr auto;
		gap: var(--space-3);
		padding: var(--space-3);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
	}
	.line-thumb {
		width: 76px;
		height: 76px;
		border-radius: var(--radius-md);
		display: flex;
		align-items: flex-end;
		padding: var(--space-2);
		overflow: hidden;
	}
	.line-kind {
		font-size: 10px;
		font-weight: var(--weight-semibold);
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: rgba(0, 0, 0, 0.7);
		background: rgba(255, 255, 255, 0.75);
		padding: 2px 6px;
		border-radius: var(--radius-xs);
		line-height: 1;
	}
	.line-body {
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.line-name {
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
		font-size: var(--text-sm);
		line-height: var(--leading-snug);
		margin: 0;
	}
	.line-sub {
		color: var(--ink-400);
		font-size: var(--text-xs);
		margin: 0;
	}
	.line-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: auto;
		gap: var(--space-3);
	}
	.qty {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		padding: 2px;
	}
	.qty button {
		width: 24px;
		height: 24px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		color: var(--ink-300);
		background: transparent;
	}
	.qty button:hover {
		background: var(--surface-3);
		color: var(--ink-100);
	}
	.qty span {
		min-width: 18px;
		text-align: center;
		font-size: var(--text-xs);
		font-weight: var(--weight-semibold);
		color: var(--ink-100);
	}
	.remove {
		color: var(--ink-400);
		display: inline-flex;
		align-items: center;
		padding: 4px;
		border-radius: var(--radius-xs);
	}
	.remove:hover {
		color: var(--danger);
		background: rgba(217, 104, 104, 0.08);
	}
	.line-price {
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		font-size: var(--text-md);
		color: var(--ink-100);
		display: flex;
		align-items: flex-start;
	}

	.drawer-footer {
		padding: var(--space-5) var(--space-6) var(--space-6);
		border-top: 1px solid var(--border-default);
		background: var(--surface-elevated);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.row-between {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}
	.totals .row-between span {
		color: var(--ink-300);
		font-size: var(--text-sm);
	}
	.totals .row-between strong {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		color: var(--ink-100);
	}
	.taxes-note {
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: var(--space-2) 0 0;
	}
	.clear-btn {
		font-size: var(--text-xs);
		color: var(--ink-400);
		text-decoration: underline;
		background: transparent;
		align-self: center;
	}
	.clear-btn:hover {
		color: var(--ink-200);
	}
</style>
