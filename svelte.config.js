import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter(),
		alias: {
			$lib: 'src/lib',
			'$lib/*': 'src/lib/*'
		},
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
