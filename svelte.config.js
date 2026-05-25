import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const isProd = process.env.NODE_ENV === 'production';

/**
 * Three transforms on every Svelte <style> block, BEFORE vitePreprocess
 * hands it to Lightning CSS:
 *
 *   1. Declare the canonical @layer priority order. Every CSS chunk
 *      the browser parses must establish the same cascade. SvelteKit
 *      can emit route-specific <link> tags BEFORE the global
 *      stylesheet (cache state, network order, HTTP/2 multiplexing),
 *      and CSS @layer ordering is set by FIRST encounter — once
 *      the browser sees `@layer components{...}` without a prior
 *      priority declaration, `components` becomes the lowest layer
 *      and any later-declared layers (reset/tokens/base/utilities)
 *      land HIGHER. Result: utility classes start beating components,
 *      layout breaks intermittently. Declaring the priority in every
 *      file (subsequent re-declarations are no-ops) prevents this.
 *
 *   2. Prepend the canonical @custom-media breakpoint declarations.
 *      Lightning CSS resolves @custom-media within a single file
 *      only, so each component needs its own copy.
 *
 *   3. Wrap the user's content in `@layer components { ... }` so
 *      component-local styles live in the components layer of the
 *      cascade declared in (1).
 *
 * Why a Svelte preprocessor and not a Vite plugin: vitePreprocess
 * runs Lightning CSS inside Svelte's preprocessor pipeline, so by
 * the time Vite's plugin transform hooks see the .svelte file, the
 * CSS has already been parsed and any unknown at-rules have already
 * errored. Injection has to happen here, ahead of vitePreprocess.
 *
 * Keep aligned with src/lib/styles/breakpoints.css (which carries
 * the same declarations for global imports) and the --bp-* CSS
 * variables in src/lib/styles/tokens.css (for JS-side reads).
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
const cssArchitecturePreprocessor = {
	name: 'tradeflex-css-architecture',
	style({ content }) {
		return {
			code: `${CSS_FOUNDATION}\n@layer components {\n${content}\n}\n`
		};
	}
};

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: [cssArchitecturePreprocessor, vitePreprocess()],
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
