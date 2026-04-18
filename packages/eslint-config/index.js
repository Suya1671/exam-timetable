import antfu from '@antfu/eslint-config'
import globals from 'globals'
import ts from 'typescript-eslint'

/**
 *
 * @type {typeof import("@antfu/eslint-config").antfu}
 */
export function examTimetable(overrides, ...extraConfigs) {
    return antfu(
        {
            ignores: ['src/lib/backend.ts'],
            ...overrides,
            typescript: {
                tsconfigPath: 'tsconfig.json',
                ...overrides.typescript,
                overrides: {
                    ...ts.configs.recommendedTypeChecked.map(config => config.rules ?? {}).reduce((a, b) => ({ ...a, ...b }), {}),
                    ...ts.configs.strictTypeChecked.map(config => config.rules ?? {}).reduce((a, b) => ({ ...a, ...b }), {}),
                    'ts/restrict-template-expressions': ['error', { allowBoolean: true, allowNumber: true }],
                    ...overrides.typescript?.overrides,
                },
            },
            formatters: {
                css: true,
                html: true,
                markdown: true,
                ...overrides.formatters,
            },
            stylistic: {
                indent: 4,
                ...overrides.stylistic,
            },
            yaml: {
                overrides: {
                    // fun fact! This doesn't work properly :)
                    'yaml/indent': 'off',
                    ...overrides.yaml?.overrides,
                },
            },
            rules: {
                'antfu/no-top-level-await': 'off',
                ...overrides.rules,
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
        ...extraConfigs,
    )
}

export default examTimetable
