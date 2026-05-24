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

export const DASHBOARD_NAV: Array<NavItem & { icon: string }> = [
	{ label: 'Overview', href: '/dashboard', icon: 'IconLayoutDashboard' },
	{ label: 'My Courses', href: '/dashboard/courses', icon: 'IconBook2' },
	{ label: 'My Indicators', href: '/dashboard/indicators', icon: 'IconChartCandle' },
	{ label: 'Subscription', href: '/dashboard/subscription', icon: 'IconStar' },
	{ label: 'Billing', href: '/dashboard/billing', icon: 'IconReceipt2' },
	{ label: 'Downloads', href: '/dashboard/downloads', icon: 'IconDownload' },
	{ label: 'Profile', href: '/dashboard/profile', icon: 'IconUserCircle' },
	{ label: 'Notifications', href: '/dashboard/notifications', icon: 'IconBell' }
];

export const ADMIN_NAV: Array<{ heading: string; items: Array<NavItem & { icon: string }> }> = [
	{
		heading: 'Catalog',
		items: [
			{ label: 'Overview', href: '/admin', icon: 'IconLayoutDashboard' },
			{ label: 'Products', href: '/admin/products', icon: 'IconBox' },
			{ label: 'Courses', href: '/admin/courses', icon: 'IconBook' },
			{ label: 'Plans', href: '/admin/plans', icon: 'IconStar' }
		]
	},
	{
		heading: 'Sales',
		items: [
			{ label: 'Orders', href: '/admin/orders', icon: 'IconShoppingBag' },
			{ label: 'Customers', href: '/admin/customers', icon: 'IconUsers' }
		]
	},
	{
		heading: 'Audience',
		items: [
			{ label: 'Leads', href: '/admin/leads', icon: 'IconUserPlus' },
			{ label: 'Messages', href: '/admin/messages', icon: 'IconMessages' }
		]
	},
	{
		heading: 'System',
		items: [{ label: 'Settings', href: '/admin/settings', icon: 'IconSettings' }]
	}
];
