import examTimetable from '@exam-timetable/eslint-config'

// For more info, see https://github.com/storybookjs/eslint-plugin-storybook#configuration-flat-config-format
import storybook from 'eslint-plugin-storybook'
import svelte from 'eslint-plugin-svelte'
import ts from 'typescript-eslint'
import svelteConfig from './svelte.config.js'

export default examTimetable(
    {
        ignores: ['!.storybook', 'vitest.shims.d.ts'],
        svelte: true,
        type: 'lib',
        typescript: {
            parserOptions: {
                projectService: {
                    allowDefaultProject: ['.storybook/*.ts', './*.js'],
                },
            },
        },
    },
    storybook.configs['flat/recommended'],
    storybook.configs['flat/addon-interactions'],
    storybook.configs['flat/csf-strict'],
    svelte.configs.recommended,
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
