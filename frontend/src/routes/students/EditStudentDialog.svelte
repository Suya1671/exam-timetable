<script lang='ts'>
    import { db } from '$lib/db'
    import { student } from '$lib/db/schema'
    import EnhancedTextInput from '$lib/EnhancedTextInput.svelte'
    import { createForm } from '@tanstack/svelte-form'
    import { eq } from 'drizzle-orm'
    import { Button, Dialog } from 'm3-svelte'
    import {
        DEFAULT_GRADE,
        gradeValidator,
        MAX_GRADE,
        MIN_GRADE,
        studentNameValidator,
    } from './forms'

    const {
        open = false,
        studentId = 0,
        name = '',
        grade = DEFAULT_GRADE,
        onClose,
        onSaved,
    }: {
        open?: boolean
        studentId?: number
        name?: string
        grade?: number
        onClose?: () => void
        onSaved?: () => void | Promise<void>
    } = $props()

    let previousOpen = $state(false)

    const editStudentForm = createForm(() => ({
        defaultValues: {
            studentId: 0,
            name: '',
            grade: DEFAULT_GRADE,
        },
        onSubmit: async ({ value, formApi }) => {
            if (!Number.isFinite(value.studentId) || value.studentId <= 0)
                return

            await db
                .update(student)
                .set({
                    name: value.name.trim(),
                    grade: value.grade,
                })
                .where(eq(student.id, value.studentId))

            await onSaved?.()
            onClose?.()
            formApi.reset()
        },
    }))

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function seedFormValues() {
        editStudentForm.setFieldValue('studentId', studentId)
        editStudentForm.setFieldValue('name', name)
        editStudentForm.setFieldValue('grade', grade)
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function handleFormSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        editStudentForm.handleSubmit()
    }

    $effect(() => {
        if (open && !previousOpen) {
            seedFormValues()
        }

        previousOpen = open
    })
</script>

<Dialog
    headline='Edit Student'
    bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
    <form id='edit-student-form' onsubmit={handleFormSubmit}>
        <editStudentForm.Field name='name' validators={{ onChange: studentNameValidator }}>
            {#snippet children(field)}
                <EnhancedTextInput {field} label='Student Name' required />
            {/snippet}
        </editStudentForm.Field>

        <editStudentForm.Field name='grade' validators={{ onChange: gradeValidator }}>
            {#snippet children(field)}
                <EnhancedTextInput
                    {field}
                    label='Grade'
                    type='number'
                    required
                    min={MIN_GRADE}
                    max={MAX_GRADE}
                />
            {/snippet}
        </editStudentForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant='outlined' onclick={onClose}>Cancel</Button>
        <Button form='edit-student-form' type='submit'>Save</Button>
    {/snippet}
</Dialog>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
</style>
