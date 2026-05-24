<script lang="ts">
	import { IconBell, IconSearch, IconChevronDown } from '@tabler/icons-svelte';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

	type Props = { user: NonNullable<App.Locals['user']>; title?: string };
	let { user, title }: Props = $props();
	let menuOpen = $state(false);

	const initials = $derived(
		user.name
			.split(' ')
			.map((p) => p[0])
			.slice(0, 2)
			.join('')
			.toUpperCase()
	);

	function defaultTitleFor(pathname: string): string {
		const map: Record<string, string> = {
			'/dashboard': 'Overview',
			'/dashboard/courses': 'My Courses',
			'/dashboard/indicators': 'My Indicators',
			'/dashboard/subscription': 'Subscription',
			'/dashboard/billing': 'Billing',
			'/dashboard/downloads': 'Downloads',
			'/dashboard/profile': 'Profile',
			'/dashboard/notifications': 'Notifications'
		};
		return map[pathname] ?? 'Dashboard';
	}

	const resolvedTitle = $derived(title ?? defaultTitleFor(page.url.pathname));
</script>

<header class="dh">
	<div class="lead">
		<h1 class="title">{resolvedTitle}</h1>
	</div>

	<div class="actions">
		<div class="search">
			<span class="search-icon"><IconSearch size={16} /></span>
			<input type="search" placeholder="Search dashboard…" aria-label="Search dashboard" />
			<kbd>⌘ K</kbd>
		</div>
		<button class="bell" type="button" aria-label="Notifications">
			<IconBell size={18} />
			<span class="dot" aria-hidden="true"></span>
		</button>
		<button
			class="user"
			type="button"
			aria-haspopup="menu"
			aria-expanded={menuOpen}
			onclick={() => (menuOpen = !menuOpen)}
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
		<a href={resolve('/dashboard/profile')} role="menuitem">Profile settings</a>
		<a href={resolve('/dashboard/subscription')} role="menuitem">Manage subscription</a>
		<a href={resolve('/dashboard/billing')} role="menuitem">Billing</a>
		<div class="m-divider"></div>
		<a href={resolve('/')} role="menuitem">Back to site</a>
		<a href={resolve('/login')} role="menuitem">Sign out</a>
	</div>
{/if}

<style>
	.dh {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-5) clamp(1rem, 3vw, 2rem);
		background: rgba(17, 17, 20, 0.8);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		border-bottom: 1px solid var(--border-default);
		position: sticky;
		top: 0;
		z-index: var(--z-sticky);
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
	.search {
		position: relative;
		display: none;
	}
	.search-icon {
		position: absolute;
		left: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		color: var(--ink-400);
	}
	.search input {
		height: 40px;
		width: 280px;
		padding: 0 var(--space-10) 0 var(--space-9);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-full);
		color: var(--ink-200);
		font-size: var(--text-sm);
		font-family: inherit;
	}
	.search input:focus {
		outline: none;
		border-color: var(--gold-500);
		background: var(--surface-1);
		box-shadow: 0 0 0 3px rgba(232, 182, 96, 0.18);
	}
	kbd {
		position: absolute;
		right: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		padding: 2px 8px;
		font-size: 10px;
		font-family: var(--font-mono);
		color: var(--ink-400);
		background: var(--surface-1);
		border: 1px solid var(--border-default);
		border-radius: var(--radius-xs);
	}
	@media (min-width: 1024px) {
		.search {
			display: block;
		}
	}

	.bell {
		position: relative;
		width: 40px;
		height: 40px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-full);
		background: var(--surface-2);
		border: 1px solid var(--border-default);
		color: var(--ink-200);
	}
	.bell:hover {
		color: var(--ink-100);
		background: var(--surface-3);
	}
	.bell .dot {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 8px;
		height: 8px;
		background: var(--gold-400);
		border-radius: 50%;
		box-shadow: 0 0 0 2px var(--surface-1);
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
	.user:hover {
		background: var(--surface-3);
		color: var(--ink-100);
	}
	.avatar {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-full);
		background: var(--gradient-gold);
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
	@media (min-width: 768px) {
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
		cursor: default;
	}
	.menu {
		position: absolute;
		right: clamp(1rem, 3vw, 2rem);
		top: calc(var(--navbar-height) + var(--space-1));
		width: 240px;
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
		transition: all var(--dur-fast) var(--ease-out);
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
