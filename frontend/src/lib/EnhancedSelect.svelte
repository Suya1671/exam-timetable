<script lang='ts'>
    import type { AnyFieldApi } from '@tanstack/svelte-form'
    import type { ComponentProps } from 'svelte'
    import type { ChangeEventHandler } from 'svelte/elements'
    import { useStore } from '@tanstack/svelte-form'
    import { Select, SelectOutlined } from 'm3-svelte'

    // This is probably the most cursed type I've written. incredible
    type Props = Omit<
        ComponentProps<typeof Select | typeof SelectOutlined>,
        'value' | 'onChange' | 'onBlur'
    > & {
        field: AnyFieldApi
        label: string
        variant?: 'filled' | 'outlined'
        required?: boolean
        helperText?: string | null
        onValueChange?: ChangeEventHandler<HTMLSelectElement>
    }

    const {
        field,
        variant = 'outlined',
        disabled = false,
        helperText = null,
        onValueChange = undefined,
        ...rest
    }: Props = $props()

    const { current: isSubmitting } = $derived(
        useStore(field.form.store, state => state.isSubmitting),
    )

    const { current: errors } = $derived(useStore(field.store, state => state.meta.errors))

    const mergedDisabled = $derived(disabled || isSubmitting)
</script>

<div>
    {#if variant === 'outlined'}
        <SelectOutlined
            bind:value={field.state.value}
            disabled={mergedDisabled}
            onchange={(value) => {
                field.handleChange(value.currentTarget.value)
                onValueChange?.(value)
            }}
            onblur={() => field.handleBlur()}
            {...rest}
        />
    {:else}
        <Select
            bind:value={field.state.value}
            disabled={mergedDisabled}
            onchange={(value) => {
                field.handleChange(value.currentTarget.value)
                onValueChange?.(value)
            }}
            onblur={() => field.handleBlur()}
            {...rest}
        />
    {/if}

    <div
        class='m3-font-body-small'
        style='padding: 0px 1rem; color: rgb(var(--m3-scheme-on-surface-variant));'
    >
        {#if errors.length > 0}
            <ul class='ma-auto pl-0 text-red'>
                <!-- eslint-disable-next-line svelte/require-each-key -->
                {#each errors as error}
                    <li>
                        {#if typeof error === 'string'}
                            {error}
                        {:else}
                            {error?.message ?? JSON.stringify(error)}
                        {/if}
                    </li>
                {/each}
            </ul>
        {:else if helperText !== null}
            {helperText}
        {/if}
    </div>
</div>
