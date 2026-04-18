<script lang='ts'>
    import { db } from '$lib/db'
    import { subject, subjectGrade } from '$lib/db/schema'
    import { EnhancedTextInput } from '@exam-timetable/ui'
    import { createForm } from '@tanstack/svelte-form'
    import { Button, Dialog } from 'm3-svelte'
    import { nonEmpty, parse, pipe, string } from 'valibot'
    import { gradesParser } from './forms'

    const {
        open = false,
        onClose,
        onSaved,
    }: {
        open?: boolean
        onClose?: () => void
        onSaved?: () => void | Promise<void>
    } = $props()

    const addSubjectForm = createForm(() => ({
        defaultValues: {
            name: '',
            grades: '',
        },
        onSubmit: async ({ value, formApi }) => {
            const parsedGrades = parse(gradesParser, value.grades)

            await db.transaction(async (tx) => {
                const [createdSubject] = await tx
                    .insert(subject)
                    .values({ name: value.name })
                    .returning({ id: subject.id })

                if (createdSubject && parsedGrades.length > 0) {
                    await tx.insert(subjectGrade).values(
                        parsedGrades.map(grade => ({
                            subjectId: createdSubject.id,
                            grade,
                        })),
                    )
                }
            })

            await onSaved?.()
            onClose?.()
            formApi.reset()
        },
    }))

    /** AI-generated (GPT-5.3-codex). */
    function handleFormSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        addSubjectForm.handleSubmit()
    }
</script>

<Dialog
    headline='Add Subject'
    bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
    <form id='add-subject-form' onsubmit={handleFormSubmit}>
        <addSubjectForm.Field
            name='name'
            validators={{
                onChange: pipe(string(), nonEmpty('Subject name is required')),
            }}
        >
            {#snippet children(field)}
                <EnhancedTextInput {field} label='Subject Name' required />
            {/snippet}
        </addSubjectForm.Field>

        <addSubjectForm.Field name='grades' validators={{ onChange: gradesParser }}>
            {#snippet children(field)}
                <EnhancedTextInput
                    {field}
                    label='Grades'
                    placeholder='e.g. 8, 9, 10'
                    required
                    helperText='Comma-separated grades to create with this subject'
                />
            {/snippet}
        </addSubjectForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant='outlined' onclick={onClose}>Cancel</Button>
        <Button form='add-subject-form' type='submit'>Add</Button>
    {/snippet}
</Dialog>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
</style>
