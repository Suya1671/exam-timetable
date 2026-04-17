import process from 'node:process'
import { sveltekit } from '@sveltejs/kit/vite'
import { Features } from 'lightningcss'
import { defineConfig } from 'vite'
import devtoolsJson from 'vite-plugin-devtools-json'
import { functionsMixins } from 'vite-plugin-functions-mixins'
import { tokenShaker } from 'vite-plugin-token-shaker'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(() => ({
    plugins: [sveltekit(), functionsMixins({ deps: ['m3-svelte'] }), tokenShaker(), devtoolsJson()],

    // Reduce the target to reduce the amount of polyfills and transforms since targetting latest webview2 and safari and webkitgtk in tauri
    // I have no clue if these are actually good values, but they work:tm: and seem to somewhat line up with 2026 baseline targets for webview2 and webkit
    build: {
        target: ['chrome123', 'safari17.4'],
        cssTarget: ['chrome123', 'safari17.4'],
    },

    css: {
        lightningcss: {
            include: 0,
            exclude: Features.LightDark | Features.ColorFunction,
        },
    },

    devtools: {
        enabled: false,
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,

    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host ? { protocol: 'ws', host, port: 1421 } : {},
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ['**/src-tauri/**'],
        },
    },
}))
