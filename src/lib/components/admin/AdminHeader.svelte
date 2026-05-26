<script lang="ts">
	import IconBell from '@tabler/icons-svelte/icons/bell';
	import IconPlus from '@tabler/icons-svelte/icons/plus';
	import IconChevronDown from '@tabler/icons-svelte/icons/chevron-down';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

	type Props = { user: NonNullable<App.Locals['user']>; title?: string };
	let { user, title }: Props = $props();
	let menuOpen = $state(false);

	function defaultTitle(pathname: string): string {
		const map: Record<string, string> = {
			'/admin': 'Operations',
			'/admin/products': 'Products',
			'/admin/courses': 'Courses',
			'/admin/plans': 'Subscription plans',
			'/admin/orders': 'Orders',
			'/admin/customers': 'Customers',
			'/admin/leads': 'Leads',
			'/admin/messages': 'Messages',
			'/admin/settings': 'Settings'
		};
		return map[pathname] ?? 'Admin';
	}

	const initials = $derived(
		user.name
			.split(' ')
			.map((p) => p[0])
			.slice(0, 2)
			.join('')
			.toUpperCase()
	);
	const resolvedTitle = $derived(title ?? defaultTitle(page.url.pathname));
</script>

<header class="ah">
	<div class="lead">
		<h1 class="title">{resolvedTitle}</h1>
		<Badge variant="danger" size="sm">Admin mode</Badge>
	</div>
	<div class="actions">
		<button class="qa" type="button">
			<IconPlus size={14} />
			Quick add
		</button>
		<button class="bell" aria-label="Notifications" type="button">
			<IconBell size={18} />
		</button>
		<button
			class="user"
			type="button"
			onclick={() => (menuOpen = !menuOpen)}
			aria-haspopup="menu"
			aria-expanded={menuOpen}
		>
			<span class="avatar">{initials}</span>
			<span class="name">{user.name}</span>
			<IconChevronDown size={14} />
		</button>
	</div>
</header>

{#if menuOpen}
	<button
		class="menu-backdrop"
		type="button"
		aria-label="Close menu"
		onclick={() => (menuOpen = false)}
	></button>
	<div class="menu" role="menu">
		<p class="m-email">{user.email}</p>
		<a href={resolve('/dashboard')} role="menuitem">Switch to member view</a>
		<a href={resolve('/admin/settings')} role="menuitem">Site settings</a>
		<div class="m-divider"></div>
		<a href={resolve('/login')} role="menuitem">Sign out</a>
	</div>
{/if}

<style>
	.ah {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-5) clamp(1rem, 3vw, 2rem);
		/* 85% opaque already; blur is polish where supported. */
		background: rgba(17, 17, 20, 0.85);
		border-bottom: 1px solid var(--border-default);
		position: sticky;
		top: 0;
		z-index: var(--z-sticky);
	}
	@supports (backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)) {
		.ah {
			backdrop-filter: blur(20px);
			-webkit-backdrop-filter: blur(20px);
		}
	}
	.lead {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.title {
		font-family: var(--font-display);
		font-size: var(--text-2xl);
		font-weight: var(--weight-semibold);
		margin: 0;
		line-height: 1;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.qa {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 0 var(--space-4);
		height: 40px;
		background: var(--gradient-gold);
		color: var(--surface-0);
		font-weight: var(--weight-semibold);
		font-size: var(--text-sm);
		border-radius: var(--radius-full);
		border: 0;
		box-shadow: 0 4px 14px rgba(212, 162, 76, 0.28);
	}
	.bell {
		width: 40px;
		height: 40px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-200);
	}
	.user {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: 4px var(--space-3) 4px 4px;
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-200);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
	}
	.avatar {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-full);
		background: var(--danger);
		color: var(--surface-0);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-weight: var(--weight-bold);
		font-size: 11px;
	}
	.name {
		display: none;
	}
	@media (--bp-md) {
		.name {
			display: inline;
		}
	}

	.menu-backdrop {
		position: fixed;
		inset: 0;
		z-index: var(--z-overlay);
		background: transparent;
		border: 0;
	}
	.menu {
		position: absolute;
		right: clamp(1rem, 3vw, 2rem);
		top: calc(var(--navbar-height) + var(--space-1));
		width: 260px;
		padding: var(--space-2);
		background: var(--surface-elevated);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-elev-3);
		z-index: calc(var(--z-overlay) + 1);
		display: flex;
		flex-direction: column;
	}
	.m-email {
		padding: var(--space-3);
		font-size: var(--text-xs);
		color: var(--ink-400);
		margin: 0;
		border-bottom: 1px solid var(--border-default);
		word-break: break-all;
	}
	.menu a {
		padding: var(--space-3);
		color: var(--ink-200);
		text-decoration: none;
		font-size: var(--text-sm);
		border-radius: var(--radius-sm);
	}
	.menu a:hover {
		background: var(--surface-2);
		color: var(--ink-100);
	}
	.m-divider {
		height: 1px;
		background: var(--border-default);
		margin: var(--space-1) 0;
	}
</style>
