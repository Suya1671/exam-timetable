import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin'
import { sveltekit } from '@sveltejs/kit/vite'
import { playwright } from '@vitest/browser-playwright'
import { defineConfig } from 'vite'
import devtoolsJson from 'vite-plugin-devtools-json'
import { functionsMixins } from 'vite-plugin-functions-mixins'
import { tokenShaker } from 'vite-plugin-token-shaker'

const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url))

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
export default defineConfig({
    plugins: [sveltekit(), functionsMixins({ deps: ['m3-svelte'] }), tokenShaker(), devtoolsJson()],
    test: {
        projects: [{
            extends: true,
            plugins: [
                // The plugin will run tests for the stories defined in your Storybook config
                // See options at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon#storybooktest
                storybookTest({
                    configDir: path.join(dirname, '.storybook'),
                }),
            ],
            test: {
                name: 'storybook',
                browser: {
                    enabled: true,
                    headless: true,
                    // @ts-expect-error this works for now... I think?
                    provider: playwright({}),
                    instances: [{
                        browser: 'chromium',
                    }],
                },
            },
        }],
    },
})
