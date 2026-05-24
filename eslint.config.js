import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import prettier from 'eslint-config-prettier';
import globals from 'globals';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default ts.config(
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs.recommended,
	prettier,
	...svelte.configs.prettier,
	{
		languageOptions: {
			globals: { ...globals.browser, ...globals.node }
		},
		rules: {
			'no-undef': 'off',
			'@typescript-eslint/no-unused-vars': [
				'warn',
				{ argsIgnorePattern: '^_', varsIgnorePattern: '^_' }
			],
			'no-restricted-imports': [
				'error',
				{
					patterns: [
						{
							group: ['*.svg', '*.svg?*'],
							message:
								'External SVG imports are forbidden. Use Tabler icons (@tabler/icons-svelte) only.'
						}
					]
				}
			]
		}
	},
	{
		files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				projectService: true,
				extraFileExtensions: ['.svelte'],
				parser: ts.parser,
				svelteConfig
			}
		}
	},
	{
		// Nav components consume hand-curated route data from $lib/data/navigation; resolve() is
		// called via a typed wrapper to satisfy the (literal-only) overload while keeping the
		// dynamic-string call site clean. The lint rule pattern-matches on `resolve()` calls and
		// cannot see through the wrapper.
		files: [
			'src/lib/components/admin/AdminSidebar.svelte',
			'src/lib/components/dashboard/DashboardSidebar.svelte',
			'src/lib/components/layout/Navbar.svelte'
		],
		rules: {
			'svelte/no-navigation-without-resolve': 'off'
		}
	},
	{
		// Button.svelte intentionally accepts polymorphic href values (internal absolute paths,
		// hash anchors, external URLs). Callers pre-resolve internal paths; Button passes through.
		files: ['src/lib/components/ui/Button.svelte'],
		rules: {
			'svelte/no-navigation-without-resolve': 'off'
		}
	},
	{
		// JsonLd injects a JSON-LD <script> via {@html}; the literal `<script>` inside the template
		// string trips the svelte ESLint parser. The runtime output is correct and HTML-escapes `<`.
		files: ['src/lib/components/seo/JsonLd.svelte'],
		rules: {
			'svelte/valid-compile': 'off'
		}
	},
	{
		ignores: [
			'build/',
			'.svelte-kit/',
			'dist/',
			'node_modules/',
			'.drizzle/',
			'drizzle/',
			'*.config.js',
			'*.config.ts'
		]
	}
);
