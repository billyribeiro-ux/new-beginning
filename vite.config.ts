import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5173,
		strictPort: false
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
