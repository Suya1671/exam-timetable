<script lang='ts'>
    import type { AnyFieldApi } from '@tanstack/svelte-form'
    import type { HTMLInputAttributes } from 'svelte/elements'
    import type { TransitionConfig } from 'svelte/transition'
    import { dateKeyUTC } from '$lib/dateKeys'
    import iconCalendar from '@ktibow/iconset-material-symbols/calendar-today-outline'
    import { useStore } from '@tanstack/svelte-form'
    import { DatePickerDocked, easeEmphasized, Icon } from 'm3-svelte'
    import { onMount } from 'svelte'

    interface Props extends HTMLInputAttributes {
        field: AnyFieldApi
        label: string
        required?: boolean
        datePickerTitle?: string
        variant?: 'filled' | 'outlined'
    }

    const {
        field,
        label,
        required = false,
        datePickerTitle = 'Pick date',
        variant = 'filled',
        ...extra
    }: Props = $props()

    const { current: isSubmitting } = $derived(
        useStore(field.form.store, state => state.isSubmitting),
    )

    const { current: errors } = $derived(useStore(field.store, state => state.meta.errors))

    const disabled = $derived(isSubmitting)

    let input = $state<HTMLInputElement>()

    const id = $props.id()
    let hasJs = $state(false)
    onMount(() => {
        hasJs = true
    })

    let picker = $state(false)

    const clickOutside = (container: Node) => {
        const handleClick = (event: Event) => {
            if (!container.contains(event.target as Node)) {
                picker = false
            }
        }
        document.addEventListener('click', handleClick, true)
        return {
            destroy() {
                document.removeEventListener('click', handleClick, true)
            },
        }
    }

    const enterExit = (_node: Node): TransitionConfig => {
        return {
            duration: 400,
            easing: easeEmphasized,
            css: (t, u) => `clip-path: inset(-100% 0 ${u * 100}% 0 round 1rem);
transform-origin: top;
transform: scaleY(${(t * 0.3 + 0.7) * 100}%);
opacity: ${Math.min(t * 3, 1)};`,
        }
    }
</script>

<div style='display: flex;flex-direction: column;gap: 0.25rem;'>
    <div
        class='m3-container'
        class:has-js={hasJs}
        class:disabled
        class:error={errors?.length !== 0}
        class:outlined={variant === 'outlined'}
        use:clickOutside
        style:--anchor-name='--{id}'
    >
        <input
            type='date'
            {disabled}
            {required}
            {id}
            {...extra}
            bind:this={input}
            name={field.name}
            bind:value={() => dateKeyUTC(field.state.value), v => (field.state.value = v)}
            oninput={e => field.handleChange(e.currentTarget?.valueAsDate)}
            onblur={field.handleBlur}
        />
        <div class='layer' aria-hidden='true'></div>
        <label for={id}>{label}</label>
        <button
            type='button'
            class='m3-layer'
            {disabled}
            title={datePickerTitle}
            onclick={() => (picker = !picker)}
        >
            <Icon icon={iconCalendar} size={24} />
        </button>
        {#if picker}
            <div class='picker' transition:enterExit>
                <DatePickerDocked
                    date={dateKeyUTC(field.state.value)}
                    clearable={!required}
                    close={() => (picker = false)}
                    setDate={(d) => {
                        if (input) {
                            input.value = d
                            input.dispatchEvent(new Event('input', { bubbles: true }))
                        }
                    // value = d;
                    }}
                />
            </div>
        {/if}
    </div>

    {#if errors?.length > 0}
        <div
            class='m3-font-body-small'
            style='padding: 0px 1rem;color: rgb(var(--m3-scheme-on-surface-variant));'
        >
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
        </div>
    {/if}
</div>

<style>
    @position-try --picker-bottom-right {
        position-area: bottom center;
        justify-self: end;
        margin-block-start: 1rem;
    }
    @position-try --picker-top-left {
        position-area: top center;
        justify-self: start;
        margin-block-end: 1rem;
    }
    @position-try --picker-top-right {
        position-area: top center;
        justify-self: end;
        margin-block-end: 1rem;
    }

    @layer tokens {
        :root {
            --m3-field-shape: var(--m3-shape-extra-small);
        }
    }
    .m3-container {
        display: inline-flex;
        position: relative;
        height: --m3-density(3.5rem);
        min-width: 15rem;
        background-color: var(--m3c-surface-container-highest);
        border-radius: var(--m3-field-shape) var(--m3-field-shape) 0 0;
        border-bottom: solid 1px var(--error, var(--m3c-on-surface-variant));
        anchor-name: var(--anchor-name);
    }

    .m3-container.outlined {
        background-color: transparent;
        border-radius: var(--m3-field-shape);
        border: none;
    }

    .m3-container.outlined label {
        top: 0;
        translate: 0 -50%;
        inset-inline-start: 0.75rem;
        background-color: var(--m3v-background, var(--m3c-surface));
        padding-inline: 0.25rem;
    }

    .m3-container.outlined input {
        padding: 1rem;
        padding-inline-start: 0.875rem;
    }

    .m3-container.outlined .layer {
        position: absolute;
        inset: 0;
        border: 1px solid var(--error, var(--m3c-outline));
        border-radius: var(--m3-field-shape);
        pointer-events: none;
    }

    .m3-container:not(.outlined) .layer {
        display: none;
    }
    input {
        font-family: var(--m3-font);
        font-size: 1rem;
        line-height: 1.5;
        /*letter-spacing: 0;*/
        font-weight: 400;
        animation: none;
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        border: none;

        padding: 1rem 1rem 0rem 1rem;
        padding-inline-start: 0.875rem;
        @supports (-moz-appearance: none) {
            padding-inline-start: 0.75rem;
        }

        &:dir(rtl) {
            text-align: right; /* work around chromium bug 41489719 */
        }

        background-color: transparent;
        color: var(--m3c-on-surface);
    }
    label {
        font-family: var(--m3-font);
        font-size: 0.75rem;
        line-height: 1.333;
        /*letter-spacing: 0.025rem;*/
        font-weight: 400;
        position: absolute;
        inset-inline-start: 1rem;
        top: 0.5rem;
        color: var(--error, var(--m3c-on-surface-variant));
        pointer-events: none;
    }

    button {
        display: none;
        position: absolute;
        padding-left: 0.75rem;
        padding-right: 0.75rem;
        height: 100%;
        inset-inline-end: 0;

        align-items: center;
        justify-content: center;
        border: none;
        background-color: transparent;
        color: var(--m3c-on-surface-variant);
        border-top-right-radius: var(--m3-field-shape);

        cursor: pointer;
    }

    .m3-container.disabled {
        background-color: --translucent(var(--m3c-on-surface), 0.04);
        border-bottom-color: --translucent(var(--m3c-on-surface), 0.38);
    }

    input:disabled,
    button:disabled {
        color: --translucent(var(--m3c-on-surface), 0.38);
        cursor: auto;
    }

    .error {
        --error: var(--m3c-error);
    }

    .picker {
        @supports not (anchor-name: --a) {
            position: absolute;
            top: calc(100% + 1rem);
            right: 0;
        }
        @supports (anchor-name: --a) {
            position: fixed;
            position-anchor: var(--anchor-name);
            /* Default */
            position-area: bottom center;
            justify-self: start;
            margin-block-start: 1rem;
            /* Alternatives */
            position-try-fallbacks: --picker-bottom-right, --picker-top-left, --picker-top-right;
        }
        z-index: 1;
    }

    @media (min-width: 37.5rem) {
        .has-js button {
            display: flex;
        }
        .has-js input {
            @supports selector(::-webkit-calendar-picker-indicator) {
                &::-webkit-calendar-picker-indicator {
                    display: none;
                }
            }
            @supports not selector(::-webkit-calendar-picker-indicator) {
                clip-path: inset(0 3.5rem 0 0);
                &:dir(rtl) {
                    clip-path: inset(0 0 0 3.5rem);
                }
            }
        }
    }
</style>
