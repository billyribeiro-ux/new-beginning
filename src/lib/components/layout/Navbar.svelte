<script lang="ts">
	import { page } from '$app/state';
	import { IconShoppingBag, IconUser, IconMenu2, IconX } from '@tabler/icons-svelte';
	import LogoWordmark from '$lib/components/media/LogoWordmark.svelte';
	import { MAIN_NAV } from '$lib/data/navigation.js';
	import { cart } from '$lib/stores/cart.svelte.js';
	import { ui } from '$lib/stores/ui.svelte.js';
	import { fly } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { browser } from '$app/environment';

	let scrolled = $state(false);

	$effect(() => {
		if (!browser) return;
		const handler = () => {
			scrolled = window.scrollY > 12;
		};
		handler();
		window.addEventListener('scroll', handler, { passive: true });
		return () => window.removeEventListener('scroll', handler);
	});

	// Pulse the cart badge on add
	let pulsing = $state(false);
	let lastToken = $state(0);
	$effect(() => {
		if (cart.pulseToken !== lastToken) {
			lastToken = cart.pulseToken;
			if (lastToken > 0) {
				pulsing = true;
				setTimeout(() => (pulsing = false), 480);
			}
		}
	});

	function isActive(href: string): boolean {
		if (href === '/') return page.url.pathname === '/';
		return page.url.pathname === href || page.url.pathname.startsWith(`${href}/`);
	}
</script>

<header class="navbar" class:is-scrolled={scrolled}>
	<div class="bar">
		<LogoWordmark href="/" size={36} />

		<nav class="main-nav" aria-label="Primary">
			<ul>
				{#each MAIN_NAV as item}
					<li>
						<a href={item.href} class:is-active={isActive(item.href)}>{item.label}</a>
					</li>
				{/each}
			</ul>
		</nav>

		<div class="actions">
			<a class="action-btn" href="/login" aria-label="Sign in or open account">
				<IconUser size={18} />
				<span class="hide-md">Account</span>
			</a>
			<button
				class="action-btn cart-btn"
				type="button"
				aria-label="Open cart ({cart.count} items)"
				onclick={() => cart.open()}
			>
				<IconShoppingBag size={18} />
				{#if cart.count > 0}
					<span class="badge" class:is-pulsing={pulsing}>{cart.count}</span>
				{/if}
			</button>
			<button
				class="action-btn mobile-toggle"
				type="button"
				aria-label={ui.mobileNavOpen ? 'Close menu' : 'Open menu'}
				aria-expanded={ui.mobileNavOpen}
				onclick={() => (ui.mobileNavOpen ? ui.closeMobileNav() : ui.openMobileNav())}
			>
				{#if ui.mobileNavOpen}<IconX size={20} />{:else}<IconMenu2 size={20} />{/if}
			</button>
		</div>
	</div>
</header>

{#if ui.mobileNavOpen}
	<div class="mobile-overlay" transition:fly={{ y: -8, duration: 220, easing: cubicOut }}>
		<nav aria-label="Mobile primary">
			<ul>
				{#each MAIN_NAV as item}
					<li>
						<a
							href={item.href}
							onclick={() => ui.closeMobileNav()}
							class:is-active={isActive(item.href)}
						>
							{item.label}
						</a>
					</li>
				{/each}
				<li class="divider-row"></li>
				<li><a href="/login" onclick={() => ui.closeMobileNav()}>Sign in</a></li>
				<li><a href="/signup" onclick={() => ui.closeMobileNav()}>Sign up</a></li>
				<li>
					<a href="/free-guide" onclick={() => ui.closeMobileNav()} class="cta-link"
						>Get the Free Greeks Guide</a
					>
				</li>
			</ul>
		</nav>
	</div>
{/if}

<style>
	.navbar {
		position: sticky;
		top: 0;
		z-index: var(--z-sticky);
		background: rgba(10, 10, 11, 0.4);
		backdrop-filter: blur(20px) saturate(140%);
		-webkit-backdrop-filter: blur(20px) saturate(140%);
		border-bottom: 1px solid transparent;
		transition:
			background var(--dur-base) var(--ease-out),
			border-color var(--dur-base) var(--ease-out),
			box-shadow var(--dur-base) var(--ease-out);
	}
	.navbar.is-scrolled {
		background: rgba(10, 10, 11, 0.82);
		border-bottom-color: var(--border-subtle);
		box-shadow: var(--shadow-elev-2);
	}
	.bar {
		max-width: var(--container-max);
		margin-inline: auto;
		padding: 0 var(--container-gutter);
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: var(--space-6);
		height: var(--navbar-height);
	}
	.main-nav {
		display: none;
		justify-content: center;
	}
	.main-nav ul {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		list-style: none;
	}
	.main-nav a {
		display: inline-block;
		padding: var(--space-2) var(--space-4);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		color: var(--ink-300);
		border-radius: var(--radius-full);
		transition: all var(--dur-fast) var(--ease-out);
	}
	.main-nav a:hover {
		color: var(--ink-100);
		background: var(--surface-2);
	}
	.main-nav a.is-active {
		color: var(--gold-300);
		background: linear-gradient(135deg, rgba(245, 208, 138, 0.12), rgba(176, 131, 47, 0.05));
		box-shadow: inset 0 0 0 1px var(--border-gold);
	}

	.actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.action-btn {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		min-width: 44px;
		height: 44px;
		padding: 0 var(--space-3);
		color: var(--ink-200);
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--radius-full);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		cursor: pointer;
		text-decoration: none;
		transition: all var(--dur-fast) var(--ease-out);
	}
	.action-btn:hover {
		color: var(--ink-100);
		background: var(--surface-2);
		border-color: var(--border-default);
	}

	.cart-btn .badge {
		position: absolute;
		top: 4px;
		right: 2px;
		min-width: 18px;
		height: 18px;
		padding: 0 5px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		font-size: 10px;
		font-weight: var(--weight-bold);
		border-radius: var(--radius-full);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		line-height: 1;
	}
	.badge.is-pulsing {
		animation: cart-bounce 0.48s var(--ease-spring);
	}
	@keyframes cart-bounce {
		0% {
			transform: scale(1);
		}
		40% {
			transform: scale(1.35);
		}
		100% {
			transform: scale(1);
		}
	}

	.hide-md {
		display: none;
	}

	.mobile-toggle {
		display: inline-flex;
	}

	@media (min-width: 1024px) {
		.main-nav {
			display: flex;
		}
		.mobile-toggle {
			display: none;
		}
		.hide-md {
			display: inline;
		}
	}

	.mobile-overlay {
		position: fixed;
		top: var(--navbar-height);
		left: 0;
		right: 0;
		z-index: var(--z-sticky);
		background: rgba(10, 10, 11, 0.96);
		backdrop-filter: blur(24px);
		-webkit-backdrop-filter: blur(24px);
		border-bottom: 1px solid var(--border-default);
		padding: var(--space-4);
	}
	.mobile-overlay ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		max-width: var(--container-max);
		margin-inline: auto;
	}
	.mobile-overlay a {
		display: block;
		padding: var(--space-4) var(--space-4);
		border-radius: var(--radius-md);
		color: var(--ink-200);
		font-size: var(--text-md);
		text-decoration: none;
	}
	.mobile-overlay a:hover {
		background: var(--surface-2);
		color: var(--ink-100);
	}
	.mobile-overlay a.is-active {
		color: var(--gold-300);
		background: var(--surface-2);
	}
	.mobile-overlay .cta-link {
		margin-top: var(--space-2);
		background: var(--gradient-gold);
		color: var(--surface-0);
		text-align: center;
		font-weight: var(--weight-semibold);
	}
	.divider-row {
		height: 1px;
		background: var(--border-default);
		margin: var(--space-2) 0;
	}
</style>
