// ESLint lints and formats JS, TS and Svelte; Biome only handles JSON, CSS
// and HTML (see ../biome.json)
import path from 'node:path';
import js from '@eslint/js';
import stylistic from '@stylistic/eslint-plugin';
import svelte from 'eslint-plugin-svelte';
import { defineConfig, includeIgnoreFile } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';

const gitignorePath = path.resolve(import.meta.dirname, '.gitignore');

/**
 * Tab indentation inside Svelte <style> blocks, which svelte/indent skips.
 * Space indents become tabs, taking the block's smallest space indent as one
 * level
 */
const styleTabs = {
	meta: {
		type: 'layout',
		fixable: 'whitespace',
		messages: { spaces: 'Indent <style> with tabs, not spaces' }
	},
	create(context) {
		return {
			SvelteStyleElement(node) {
				const text = context.sourceCode.getText(node);
				const lines = text.split('\n');
				const indents = lines
					.slice(1)
					.map((line) => /^(\t*)( +)\S/.exec(line))
					.filter(Boolean);
				if (!indents.length) return;
				const unit = Math.min(...indents.map((m) => m[2].length));
				let offset = node.range[0] + lines[0].length + 1;
				for (const line of lines.slice(1)) {
					const m = /^(\t*)( +)\S/.exec(line);
					if (m) {
						const range = [offset, offset + m[1].length + m[2].length];
						const tabs = '\t'.repeat(m[1].length + Math.round(m[2].length / unit));
						context.report({
							loc: context.sourceCode.getLocFromIndex(offset),
							messageId: 'spaces',
							fix: (fixer) => fixer.replaceTextRange(range, tabs)
						});
					}
					offset += line.length + 1;
				}
			}
		};
	}
};

export default defineConfig(
	includeIgnoreFile(gitignorePath),
	{ ignores: ['static/**', 'worker-configuration.d.ts'] },
	js.configs.recommended,
	ts.configs.recommended,
	svelte.configs.recommended,
	{
		languageOptions: { globals: { ...globals.browser, ...globals.node } },
		rules: {
			// typescript-eslint recommends leaving undefined globals to TypeScript:
			// https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
			'no-undef': 'off',
			// A leading underscore marks a parameter a signature needs but the body doesn't
			'@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }]
		}
	},
	{
		files: ['**/*.svelte', '**/*.svelte.{js,ts}'],
		languageOptions: {
			parserOptions: {
				projectService: true,
				extraFileExtensions: ['.svelte'],
				parser: ts.parser
			}
		}
	},
	// Formatting
	{
		files: ['**/*.{js,ts,svelte}'],
		plugins: { '@stylistic': stylistic },
		rules: {
			...stylistic.configs.customize({
				indent: 'tab',
				quotes: 'single',
				semi: true,
				commaDangle: 'never',
				braceStyle: '1tbs',
				arrowParens: true,
				jsx: false
			}).rules,
			// Keep the Prettier-era layout the code already has
			'@stylistic/quotes': ['error', 'single', { avoidEscape: true }],
			'@stylistic/quote-props': ['error', 'as-needed'],
			'@stylistic/operator-linebreak': [
				'error',
				'after',
				{ overrides: { '?': 'before', ':': 'before', '|': 'before' } }
			],
			// Tabs indent, spaces align comment continuations
			'@stylistic/no-mixed-spaces-and-tabs': ['error', 'smart-tabs']
		}
	},
	{
		files: ['**/*.svelte'],
		plugins: { local: { rules: { 'style-tabs': styleTabs } } },
		rules: {
			// svelte/indent handles <script> and markup together
			'@stylistic/indent': 'off',
			'@stylistic/indent-binary-ops': 'off',
			'svelte/indent': ['error', { indent: 'tab', indentScript: true }],
			'local/style-tabs': 'error',
			'svelte/html-closing-bracket-spacing': 'error',
			'svelte/html-quotes': 'error',
			'svelte/mustache-spacing': 'error',
			'svelte/no-spaces-around-equal-signs-in-attribute': 'error',
			'svelte/no-trailing-spaces': 'error',
			'svelte/spaced-html-comment': 'error'
		}
	}
);
