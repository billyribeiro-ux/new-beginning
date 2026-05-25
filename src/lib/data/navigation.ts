/* Tabler exports its own canonical icon component type. Using the
 * package's own type avoids structural-typing mismatches between
 * Svelte 5's `Component` and Tabler's Svelte 4-style classes. */
import type { Icon as IconComponent } from '@tabler/icons-svelte';

import IconLayoutDashboard from '@tabler/icons-svelte/icons/layout-dashboard';
import IconBook from '@tabler/icons-svelte/icons/book';
import IconBook2 from '@tabler/icons-svelte/icons/book-2';
import IconBox from '@tabler/icons-svelte/icons/box';
import IconChartCandle from '@tabler/icons-svelte/icons/chart-candle';
import IconStar from '@tabler/icons-svelte/icons/star';
import IconReceipt2 from '@tabler/icons-svelte/icons/receipt-2';
import IconDownload from '@tabler/icons-svelte/icons/download';
import IconUserCircle from '@tabler/icons-svelte/icons/user-circle';
import IconBell from '@tabler/icons-svelte/icons/bell';
import IconShoppingBag from '@tabler/icons-svelte/icons/shopping-bag';
import IconUsers from '@tabler/icons-svelte/icons/users';
import IconUserPlus from '@tabler/icons-svelte/icons/user-plus';
import IconMessages from '@tabler/icons-svelte/icons/messages';
import IconSettings from '@tabler/icons-svelte/icons/settings';

export interface NavItem {
	label: string;
	href: string;
	external?: boolean;
}

export interface NavSection {
	heading: string;
	items: NavItem[];
}

export const MAIN_NAV: NavItem[] = [
	{ label: 'Subscription', href: '/subscription' },
	{ label: 'Indicators', href: '/indicators' },
	{ label: 'Courses', href: '/courses' },
	{ label: 'About', href: '/about' },
	{ label: 'Contact', href: '/contact' }
];

export const FOOTER_NAV: NavSection[] = [
	{
		heading: 'Products',
		items: [
			{ label: 'Day Trading Subscription', href: '/subscription' },
			{ label: 'Revolution Ranger', href: '/indicators/revolution-ranger' },
			{ label: 'Options 101', href: '/courses/options-101' },
			{ label: 'Free Options Greeks Guide', href: '/free-guide' }
		]
	},
	{
		heading: 'Company',
		items: [
			{ label: 'About', href: '/about' },
			{ label: 'Contact', href: '/contact' },
			{ label: 'Login', href: '/login' },
			{ label: 'Sign up', href: '/signup' }
		]
	},
	{
		heading: 'Legal',
		items: [
			{ label: 'Terms of Service', href: '/legal/terms' },
			{ label: 'Privacy Policy', href: '/legal/privacy' },
			{ label: 'Refund Policy', href: '/legal/refund' }
		]
	}
];

/* Icon field is the statically-imported component itself (not a string).
 * This lets the bundler tree-shake everything else; the alternative
 * (string lookup against `import * as TablerIcons`) forces the whole
 * icon library into every bundle that touches these arrays. */
export const DASHBOARD_NAV: Array<NavItem & { icon: IconComponent }> = [
	{ label: 'Overview', href: '/dashboard', icon: IconLayoutDashboard },
	{ label: 'My Courses', href: '/dashboard/courses', icon: IconBook2 },
	{ label: 'My Indicators', href: '/dashboard/indicators', icon: IconChartCandle },
	{ label: 'Subscription', href: '/dashboard/subscription', icon: IconStar },
	{ label: 'Billing', href: '/dashboard/billing', icon: IconReceipt2 },
	{ label: 'Downloads', href: '/dashboard/downloads', icon: IconDownload },
	{ label: 'Profile', href: '/dashboard/profile', icon: IconUserCircle },
	{ label: 'Notifications', href: '/dashboard/notifications', icon: IconBell }
];

export const ADMIN_NAV: Array<{
	heading: string;
	items: Array<NavItem & { icon: IconComponent }>;
}> = [
	{
		heading: 'Catalog',
		items: [
			{ label: 'Overview', href: '/admin', icon: IconLayoutDashboard },
			{ label: 'Products', href: '/admin/products', icon: IconBox },
			{ label: 'Courses', href: '/admin/courses', icon: IconBook },
			{ label: 'Plans', href: '/admin/plans', icon: IconStar }
		]
	},
	{
		heading: 'Sales',
		items: [
			{ label: 'Orders', href: '/admin/orders', icon: IconShoppingBag },
			{ label: 'Customers', href: '/admin/customers', icon: IconUsers }
		]
	},
	{
		heading: 'Audience',
		items: [
			{ label: 'Leads', href: '/admin/leads', icon: IconUserPlus },
			{ label: 'Messages', href: '/admin/messages', icon: IconMessages }
		]
	},
	{
		heading: 'System',
		items: [{ label: 'Settings', href: '/admin/settings', icon: IconSettings }]
	}
];
