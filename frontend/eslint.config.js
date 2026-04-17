import antfu from '@antfu/eslint-config'
import drizzle from 'eslint-plugin-drizzle'
import svelte from 'eslint-plugin-svelte'
import globals from 'globals'
import ts from 'typescript-eslint'
import svelteConfig from './svelte.config.js'

export default antfu(
    {
        ignores: ['src/lib/backend.ts'],
        svelte: true,
        typescript: {
            tsconfigPath: 'tsconfig.json',
            overrides: {
                ...ts.configs.recommendedTypeChecked.map(config => config.rules ?? {}).reduce((a, b) => ({ ...a, ...b }), {}),
                ...ts.configs.strictTypeChecked.map(config => config.rules ?? {}).reduce((a, b) => ({ ...a, ...b }), {}),
                'ts/restrict-template-expressions': ['error', { allowBoolean: true, allowNumber: true }],
            },
        },
        formatters: {
            css: true,
            html: true,
            markdown: true,
        },
        stylistic: {
            indent: 4,
        },
        yaml: {
            overrides: {
                // fun fact! This doesn't work properly :)
                'yaml/indent': 'off',
            },
        },
        rules: {
            'antfu/no-top-level-await': 'off',
        },
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
        languageOptions: { globals: { ...globals.browser, ...globals.node } },
        rules: {
            // typescript-eslint strongly recommend that you do not use the no-undef lint rule on TypeScript projects.
            // see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
            'no-undef': 'off',
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
