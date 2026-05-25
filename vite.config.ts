import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		strictPort: false
	},
	css: {
		// Route CSS through Lightning CSS as the transformer so the
		// @custom-media draft (Media Queries Level 5) resolves at
		// build time. Without `transformer: 'lightningcss'` the
		// minifier-only mode runs too late in the pipeline and
		// `@media (--bp-md)` references never expand.
		//
		// Side effect: Lightning CSS now sees Svelte's `:global()`
		// pseudo-class during preprocessing and logs ~60 "global
		// pseudo-class not recognized" notices. These are cosmetic —
		// Svelte's own compiler strips :global() correctly before
		// the CSS reaches the browser; the built CSS has zero
		// :global() references. Cannot suppress without forking
		// Vite's plugin pipeline; living with the noise.
		transformer: 'lightningcss',
		lightningcss: {
			drafts: {
				customMedia: true
			}
		}
	},
	build: {
		target: 'es2022',
		cssMinify: 'lightningcss',
		sourcemap: true
	},
	ssr: {
		noExternal: ['gsap']
	}
});
