<script lang="ts">
	import { page } from '$app/state';
	import * as TablerIcons from '@tabler/icons-svelte';
	import type { Component } from 'svelte';
	import {
		IconLayoutSidebarLeftCollapse,
		IconLayoutSidebarLeftExpand,
		IconLogout,
		IconCircleDot
	} from '@tabler/icons-svelte';
	import LogoMark from '$lib/components/media/LogoMark.svelte';
	import { DASHBOARD_NAV } from '$lib/data/navigation.js';
	import { ui } from '$lib/stores/ui.svelte.js';
	import { resolve } from '$app/paths';

	// resolve() is typed for literal-string args; this wrapper commits to the no-params overload
	// for dynamic nav data. Safe: $lib/data/navigation is hand-curated to real routes.
	const resolveDynamic = resolve as (href: string) => string;

	function iconOf(name: string): Component {
		const dict = TablerIcons as unknown as Record<string, Component | undefined>;
		return dict[name] ?? (IconCircleDot as unknown as Component);
	}
	function isActive(href: string): boolean {
		const pathname = page.url.pathname as string;
		if (href === '/dashboard') return pathname === href;
		return pathname === href || pathname.startsWith(`${href}/`);
	}
</script>

<aside class="sidebar" class:is-collapsed={ui.sidebarCollapsed} aria-label="Dashboard navigation">
	<div class="brand">
		<a href={resolve('/')} class="brand-link" aria-label="Home">
			<LogoMark size={32} />
			{#if !ui.sidebarCollapsed}
				<span class="brand-text">
					TradeFlex
					<span class="ind">Member area</span>
				</span>
			{/if}
		</a>
	</div>

	<nav class="nav">
		<ul>
			{#each DASHBOARD_NAV as item (item.href)}
				{@const Icon = iconOf(item.icon)}
				<li>
					<a
						href={resolveDynamic(item.href)}
						class="nav-link"
						class:is-active={isActive(item.href)}
						aria-current={isActive(item.href) ? 'page' : undefined}
					>
						<span class="ic"><Icon size={18} /></span>
						{#if !ui.sidebarCollapsed}<span class="lbl">{item.label}</span>{/if}
					</a>
				</li>
			{/each}
		</ul>
	</nav>

	<div class="foot">
		<button
			class="collapse-btn"
			type="button"
			onclick={() => ui.toggleSidebar()}
			aria-label={ui.sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
		>
			{#if ui.sidebarCollapsed}<IconLayoutSidebarLeftExpand
					size={18}
				/>{:else}<IconLayoutSidebarLeftCollapse size={18} />{/if}
			{#if !ui.sidebarCollapsed}<span>Collapse</span>{/if}
		</button>
		<a class="logout" href={resolve('/login')}>
			<IconLogout size={18} />
			{#if !ui.sidebarCollapsed}<span>Sign out</span>{/if}
		</a>
	</div>
</aside>

<style>
	.sidebar {
		position: sticky;
		top: 0;
		height: 100dvh;
		width: var(--sidebar-width);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		padding: var(--space-5);
		background: var(--surface-1);
		border-right: 1px solid var(--border-default);
		transition: width var(--dur-base) var(--ease-out);
		flex-shrink: 0;
	}
	.sidebar.is-collapsed {
		width: var(--sidebar-collapsed-width);
		padding-inline: var(--space-3);
	}
	.brand {
		padding-bottom: var(--space-4);
		border-bottom: 1px solid var(--border-default);
	}
	.brand-link {
		display: inline-flex;
		align-items: center;
		gap: var(--space-3);
		text-decoration: none;
		color: var(--ink-100);
	}
	.brand-text {
		display: flex;
		flex-direction: column;
		line-height: 1;
		font-family: var(--font-display);
		font-weight: var(--weight-semibold);
		font-size: var(--text-md);
	}
	.ind {
		margin-top: 4px;
		font-family: var(--font-body);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: var(--tracking-widest);
		color: var(--gold-400);
		font-weight: var(--weight-semibold);
	}

	.nav {
		flex: 1;
		overflow-y: auto;
		min-height: 0;
	}
	.nav ul {
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.nav-link {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3);
		color: var(--ink-300);
		text-decoration: none;
		border-radius: var(--radius-md);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		transition: all var(--dur-fast) var(--ease-out);
		position: relative;
	}
	.is-collapsed .nav-link {
		justify-content: center;
		padding: var(--space-3) 0;
	}
	.nav-link:hover {
		background: var(--surface-2);
		color: var(--ink-100);
	}
	.nav-link.is-active {
		background: linear-gradient(135deg, rgba(232, 182, 96, 0.14), rgba(176, 131, 47, 0.04));
		color: var(--gold-300);
		box-shadow: inset 0 0 0 1px var(--border-gold);
	}
	.nav-link.is-active::before {
		content: '';
		position: absolute;
		left: -5px;
		top: 50%;
		transform: translateY(-50%);
		width: 3px;
		height: 18px;
		background: var(--gradient-gold);
		border-radius: var(--radius-full);
	}
	.ic {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		flex-shrink: 0;
	}

	.foot {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-default);
	}
	.collapse-btn,
	.logout {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3);
		color: var(--ink-400);
		font-size: var(--text-sm);
		font-weight: var(--weight-medium);
		background: transparent;
		border: 0;
		border-radius: var(--radius-md);
		cursor: pointer;
		text-decoration: none;
	}
	.is-collapsed .collapse-btn,
	.is-collapsed .logout {
		justify-content: center;
		padding: var(--space-3) 0;
	}
	.collapse-btn:hover,
	.logout:hover {
		background: var(--surface-2);
		color: var(--ink-100);
	}

	@media (max-width: 1023px) {
		.sidebar {
			position: fixed;
			top: 0;
			left: 0;
			z-index: var(--z-overlay);
			transform: translateX(-100%);
			transition: transform var(--dur-base) var(--ease-out);
		}
	}
</style>
