<script module>
    import { defineMeta } from '@storybook/addon-svelte-csf'
    import EnhancedDateField from './EnhancedDateField.svelte'

    const { Story } = defineMeta({
        component: EnhancedDateField,
        tags: ['autodocs'],
        argTypes: {
            label: { control: 'text' },
            required: { control: 'boolean' },
            datePickerTitle: { control: 'text' },
            variant: {
                control: { type: 'select' },
                options: ['filled', 'outlined'],
            },
        },
    })
</script>

<script lang='ts'>
    import type { ComponentProps } from 'svelte'
    import { createForm } from '@tanstack/svelte-form'

    const form = createForm(() => ({
        defaultValues: {
            date: new Date(),
        },
    }))
</script>

<Story name='Default'>
    {#snippet template(args: ComponentProps<typeof EnhancedDateField>)}
        <form.Field name='date'>
            {#snippet children(field)}
                <EnhancedDateField {...args} {field} />
            {/snippet}
        </form.Field>
    {/snippet}
</Story>
