import devtoolsJson from "vite-plugin-devtools-json";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { functionsMixins } from "vite-plugin-functions-mixins";
import { tokenShaker } from "vite-plugin-token-shaker";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [
        sveltekit(),
        functionsMixins({ deps: ["m3-svelte"] }),
        tokenShaker(),
        devtoolsJson(),
    ],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,

    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
}));
