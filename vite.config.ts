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
		// Svelte's `:global(...)` syntax — which Lightning CSS would
		// otherwise warn about ~60 times per build — is masked to a
		// custom pseudo-class `:--svelte-global(...)` by the Svelte
		// pre-preprocessor in svelte.config.js, then restored by the
		// post-preprocessor before Svelte's scoping compiler runs.
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
