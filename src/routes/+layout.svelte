<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import Navbar from '$lib/components/layout/Navbar.svelte';
	import Footer from '$lib/components/layout/Footer.svelte';
	import AnnouncementBar from '$lib/components/layout/AnnouncementBar.svelte';
	import CartDrawer from '$lib/components/layout/CartDrawer.svelte';
	import Toast from '$lib/components/ui/Toast.svelte';
	import JsonLd from '$lib/components/seo/JsonLd.svelte';
	import { organizationLd, websiteLd } from '$lib/utils/jsonld.js';

	type Props = { children: Snippet };
	let { children }: Props = $props();

	// Hide app shell on auth + dashboard + admin routes (they have their own chrome)
	const isBareRoute = $derived(
		page.url.pathname.startsWith('/dashboard') ||
			page.url.pathname.startsWith('/admin') ||
			page.url.pathname === '/login' ||
			page.url.pathname === '/signup' ||
			page.url.pathname === '/forgot-password' ||
			page.url.pathname === '/reset-password'
	);
</script>

<JsonLd data={[organizationLd(), websiteLd()]} />

{#if !isBareRoute}
	<AnnouncementBar />
	<Navbar />
	<main id="main-content">
		{@render children()}
	</main>
	<Footer />
{:else}
	{@render children()}
{/if}

<CartDrawer />
<Toast />

<style>
	main {
		min-height: calc(100dvh - var(--navbar-height) - var(--announcement-height));
	}
</style>
