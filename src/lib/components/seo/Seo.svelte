<script lang="ts">
	import { page } from '$app/state';
	import {
		SITE_NAME,
		SITE_TAGLINE,
		SITE_URL,
		TITLE_TEMPLATE,
		absoluteUrl,
		defaultOgImage
	} from '$lib/utils/seo.js';

	type Props = {
		title?: string;
		description?: string;
		canonical?: string;
		image?: string;
		noindex?: boolean;
		keywords?: string[];
		type?: 'website' | 'article' | 'product';
		publishedAt?: string;
		updatedAt?: string;
		author?: string;
	};

	let {
		title,
		description = SITE_TAGLINE,
		canonical,
		image,
		noindex = false,
		keywords,
		type = 'website',
		publishedAt,
		updatedAt,
		author
	}: Props = $props();

	const fullTitle = $derived(TITLE_TEMPLATE(title ?? ''));
	const canonicalUrl = $derived(
		canonical ? absoluteUrl(canonical) : absoluteUrl(page.url.pathname)
	);
	const ogImage = $derived(image ? absoluteUrl(image) : defaultOgImage());
</script>

<svelte:head>
	<title>{fullTitle}</title>
	<meta name="description" content={description} />
	<link rel="canonical" href={canonicalUrl} />
	{#if keywords?.length}<meta name="keywords" content={keywords.join(', ')} />{/if}
	{#if noindex}
		<meta name="robots" content="noindex, nofollow" />
	{:else}
		<meta
			name="robots"
			content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1"
		/>
	{/if}

	<!-- Open Graph -->
	<meta property="og:type" content={type} />
	<meta property="og:site_name" content={SITE_NAME} />
	<meta property="og:title" content={fullTitle} />
	<meta property="og:description" content={description} />
	<meta property="og:url" content={canonicalUrl} />
	<meta property="og:image" content={ogImage} />
	<meta property="og:image:width" content="1200" />
	<meta property="og:image:height" content="630" />
	<meta property="og:locale" content="en_US" />
	{#if publishedAt}<meta property="article:published_time" content={publishedAt} />{/if}
	{#if updatedAt}<meta property="article:modified_time" content={updatedAt} />{/if}
	{#if author}<meta name="author" content={author} />{/if}

	<!-- Twitter / X -->
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:site" content="@tradeflextrading" />
	<meta name="twitter:title" content={fullTitle} />
	<meta name="twitter:description" content={description} />
	<meta name="twitter:image" content={ogImage} />

	<!-- AI-search / 2026 signals -->
	<meta name="application-name" content={SITE_NAME} />
	<meta name="apple-mobile-web-app-title" content={SITE_NAME} />

	{#if SITE_URL}<link rel="alternate" hreflang="en" href={canonicalUrl} />{/if}
</svelte:head>
