import { sveltekit } from '@sveltejs/kit/vite'
import { Features } from 'lightningcss'
import { defineConfig } from 'vite'
import devtoolsJson from 'vite-plugin-devtools-json'
import { functionsMixins } from 'vite-plugin-functions-mixins'
import { tokenShaker } from 'vite-plugin-token-shaker'

export default defineConfig({
    plugins: [sveltekit(), functionsMixins({ deps: ['m3-svelte', '@exam-timetable/ui'] }), tokenShaker(), devtoolsJson()],

    css: {
        lightningcss: {
            include: 0,
            exclude: Features.LightDark | Features.ColorFunction,
        },
    },
})
