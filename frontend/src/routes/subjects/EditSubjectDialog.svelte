<script lang='ts'>
    import { db } from '$lib/db'
    import { subject } from '$lib/db/schema'
    import EnhancedTextInput from '$lib/EnhancedTextInput.svelte'
    import { createForm } from '@tanstack/svelte-form'
    import { eq } from 'drizzle-orm'
    import { Button, Dialog } from 'm3-svelte'
    import { nonEmpty, pipe, string } from 'valibot'

    const {
        open = false,
        subjectId = 0,
        name = '',
        onClose,
        onSaved,
    }: {
        open?: boolean
        subjectId?: number
        name?: string
        onClose?: () => void
        onSaved?: () => void | Promise<void>
    } = $props()

    let previousOpen = $state(false)

    const editSubjectForm = createForm(() => ({
        defaultValues: {
            subjectId: 0,
            name: '',
        },
        onSubmit: async ({ value, formApi }) => {
            await db
                .update(subject)
                .set({ name: value.name })
                .where(eq(subject.id, value.subjectId))

            await onSaved?.()
            onClose?.()
            formApi.reset()
        },
    }))

    /** AI-generated (GPT-5.3-codex). */
    function handleFormSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        editSubjectForm.handleSubmit()
    }

    $effect(() => {
        if (open && !previousOpen) {
            editSubjectForm.setFieldValue('subjectId', subjectId)
            editSubjectForm.setFieldValue('name', name)
        }

        previousOpen = open
    })
</script>

<Dialog
    headline='Edit Subject'
    bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
    <form id='edit-subject-form' onsubmit={handleFormSubmit}>
        <editSubjectForm.Field
            name='name'
            validators={{
                onChange: pipe(string(), nonEmpty('Subject must have a name')),
            }}
        >
            {#snippet children(field)}
                <EnhancedTextInput {field} label='Name' required />
            {/snippet}
        </editSubjectForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant='outlined' onclick={onClose}>Cancel</Button>
        <Button form='edit-subject-form' type='submit'>Edit</Button>
    {/snippet}
</Dialog>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
</style>
