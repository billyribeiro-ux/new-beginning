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
