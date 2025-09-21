import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'node:path';

const SRC_DIR = resolve(import.meta.dirname, './src');

export default defineConfig(configEnv => ({
	build: {
		modulePreload: { polyfill: false },
		sourcemap: configEnv.mode === 'development'
	},
	resolve: {
		alias: {
			'@components': resolve(SRC_DIR, './components')
		}
	},
	appType: 'spa',
	plugins: [svelte(), tailwindcss()]
}));
