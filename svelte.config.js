import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const isProd = process.env.NODE_ENV === 'production';

/**
 * CSS pipeline for every Svelte <style> block. Runs as two preprocessors
 * straddling vitePreprocess (which is where Lightning CSS lives):
 *
 *   pre  → vitePreprocess (Lightning CSS) → post → Svelte compile
 *
 * Pre-pass (`cssPrePreprocessor`):
 *   1. Inject the canonical @layer priority declaration. Every CSS chunk
 *      the browser parses must establish the same cascade — SvelteKit
 *      can emit route-specific <link> tags BEFORE the global stylesheet
 *      (cache state, network order, HTTP/2 multiplexing), and CSS @layer
 *      ordering is set by FIRST encounter. Without this in every chunk,
 *      `@layer components{...}` arrives first, becomes the lowest-priority
 *      layer, and utilities then beat component-local declarations.
 *   2. Inject the canonical @custom-media breakpoint declarations.
 *      Lightning CSS resolves @custom-media within a single file only.
 *   3. Wrap user content in `@layer components { ... }`.
 *   4. Rewrite Svelte's `:global(...)` syntax to a custom pseudo-class
 *      `:--svelte-global(...)` that Lightning CSS accepts silently —
 *      Lightning CSS warns ~60 times per build on unrecognized `:global`,
 *      and the post-pass restores the original syntax before Svelte's
 *      scoping compiler runs.
 *
 * Post-pass (`cssPostPreprocessor`):
 *   • Rewrite `:--svelte-global(...)` back to `:global(...)` so Svelte's
 *     scoping pass produces correctly-unscoped selectors in the output.
 *
 * Why a Svelte preprocessor and not a Vite plugin: vitePreprocess runs
 * Lightning CSS inside Svelte's preprocessor pipeline, so Vite's own
 * plugin transform hooks see the .svelte file AFTER Lightning CSS has
 * already parsed it. Injection has to happen here.
 *
 * Keep CSS_FOUNDATION aligned with src/lib/styles/breakpoints.css
 * (which carries the same declarations for global imports) and the
 * --bp-* CSS variables in src/lib/styles/tokens.css (for JS-side reads).
 */
const CSS_FOUNDATION = `
@layer reset, tokens, base, utilities, components, overrides;

@custom-media --bp-sm (min-width: 480px);
@custom-media --bp-md (min-width: 768px);
@custom-media --bp-lg (min-width: 1024px);
@custom-media --bp-xl (min-width: 1280px);
@custom-media --bp-2xl (min-width: 1536px);
@custom-media --bp-3xl (min-width: 1920px);
@custom-media --bp-4xl (min-width: 2560px);
@custom-media --bp-sm-down (max-width: 479.98px);
@custom-media --bp-md-down (max-width: 767.98px);
@custom-media --bp-lg-down (max-width: 1023.98px);
`;

/** @type {import('svelte/compiler').PreprocessorGroup} */
const cssPrePreprocessor = {
	name: 'tradeflex-css-pre',
	style({ content }) {
		const masked = content.replace(/:global\b/g, ':--svelte-global');
		return {
			code: `${CSS_FOUNDATION}\n@layer components {\n${masked}\n}\n`
		};
	}
};

/** @type {import('svelte/compiler').PreprocessorGroup} */
const cssPostPreprocessor = {
	name: 'tradeflex-css-post',
	style({ content }) {
		return {
			code: content.replace(/:--svelte-global\b/g, ':global')
		};
	}
};

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: [cssPrePreprocessor, vitePreprocess(), cssPostPreprocessor],
	kit: {
		adapter: adapter(),
		alias: {
			$lib: 'src/lib',
			'$lib/*': 'src/lib/*'
		},
		...(isProd && {
			csp: {
				mode: 'auto',
				directives: {
					'default-src': ['self'],
					'script-src': ['self'],
					'style-src': ['self', 'unsafe-inline', 'https://fonts.googleapis.com'],
					'img-src': ['self', 'data:', 'https:'],
					'font-src': ['self', 'data:', 'https://fonts.gstatic.com'],
					'connect-src': ['self'],
					'frame-ancestors': ['self'],
					'base-uri': ['self'],
					'form-action': ['self'],
					'object-src': ['none']
				}
			}
		}),
		typescript: {
			config: (cfg) => {
				cfg.include = [...(cfg.include ?? []), '../drizzle.config.ts'];
				return cfg;
			}
		}
	}
	// Svelte 5 auto-detects runes mode per file. We do NOT force `runes: true`
	// globally because node_modules deps (e.g. @tabler/icons-svelte) still ship
	// Svelte 4 legacy components using $$props.
};

export default config;
