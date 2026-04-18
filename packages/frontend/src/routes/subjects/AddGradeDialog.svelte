<script lang='ts'>
    import { db } from '$lib/db'
    import { subjectGrade } from '$lib/db/schema'
    import EnhancedTextInput from '$lib/EnhancedTextInput.svelte'
    import { createForm } from '@tanstack/svelte-form'
    import { Button, Dialog } from 'm3-svelte'
    import { parse } from 'valibot'
    import { gradesParser } from './forms'

    const {
        open = false,
        subjectId = 0,
        onClose,
        onSaved,
    }: {
        open?: boolean
        subjectId?: number
        onClose?: () => void
        onSaved?: () => void | Promise<void>
    } = $props()

    let previousOpen = $state(false)

    const addGradeForm = createForm(() => ({
        defaultValues: {
            subjectId: 0,
            grades: '',
        },
        onSubmit: async ({ value, formApi }) => {
            const parsedGrades = parse(gradesParser, value.grades)

            if (value.subjectId && parsedGrades.length > 0) {
                await db.insert(subjectGrade).values(
                    parsedGrades.map(grade => ({
                        subjectId: value.subjectId,
                        grade,
                    })),
                )
            }

            await onSaved?.()
            onClose?.()
            formApi.reset()
        },
    }))

    /** AI-generated (GPT-5.3-codex). */
    function handleFormSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        addGradeForm.handleSubmit()
    }

    $effect(() => {
        if (open && !previousOpen) {
            addGradeForm.setFieldValue('subjectId', subjectId)
        }

        previousOpen = open
    })
</script>

<Dialog
    headline='Add Grade'
    bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
    <form id='add-grade-form' onsubmit={handleFormSubmit}>
        <addGradeForm.Field name='grades' validators={{ onChange: gradesParser }}>
            {#snippet children(field)}
                <EnhancedTextInput
                    {field}
                    label='Grades'
                    placeholder='e.g. 8, 9, 10'
                    required
                    helperText='Comma-separated grades to add'
                />
            {/snippet}
        </addGradeForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant='outlined' onclick={onClose}>Cancel</Button>
        <Button form='add-grade-form' type='submit'>Add</Button>
    {/snippet}
</Dialog>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
</style>
