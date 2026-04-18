import examTimetable from '@exam-timetable/eslint-config'
import drizzle from 'eslint-plugin-drizzle'
import svelte from 'eslint-plugin-svelte'
import ts from 'typescript-eslint'
import svelteConfig from './svelte.config.js'

export default examTimetable(
    {
        ignores: ['src/lib/backend.ts'],
        svelte: true,
    },
    svelte.configs.recommended,
    // https://github.com/drizzle-team/drizzle-orm/issues/2491
    {
        plugins: {
            drizzle,
        },
        rules: {
            ...drizzle.configs.recommended.rules,
        },
    },
    {
        files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
        languageOptions: {
            parserOptions: {
                projectService: true,
                extraFileExtensions: ['.svelte'],
                parser: ts.parser,
                svelteConfig,
            },
        },
    },
)
