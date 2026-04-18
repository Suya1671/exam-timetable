<script lang='ts'>
    import { db } from '$lib/db'
    import { exam, examTimeslotRestriction, timeslot } from '$lib/db/schema'
    import TimeslotPicker from '$lib/TimeslotPicker.svelte'
    import { EnhancedTextInput } from '@exam-timetable/ui'
    import { createForm } from '@tanstack/svelte-form'
    import { eq } from 'drizzle-orm'
    import { Button, Dialog } from 'm3-svelte'
    import { SvelteSet } from 'svelte/reactivity'
    import { string } from 'valibot'
    import { durationHoursValidator, priorityValidator, slotsRequiredValidator } from './forms'

    type RestrictionMode = 'allow' | 'deny'
    type EditAction = 'edit' | 'delete'

    const {
        open = false,
        examId = 0,
        durationHours = 2,
        slotsRequired = 1,
        priority = 0,
        name = null,
        allTimeslots,
        onClose,
        onSaved,
    }: {
        open?: boolean
        examId?: number
        durationHours?: number
        slotsRequired?: number
        priority?: number
        name?: string | null
        allTimeslots: Array<typeof timeslot.$inferSelect>
        onClose?: () => void
        onSaved?: () => void | Promise<void>
    } = $props()

    const restrictionIds = new SvelteSet<number>()
    let restrictionMode = $state<RestrictionMode>('deny')
    let previousOpen = $state(false)

    const editExamForm = createForm(() => ({
        defaultValues: {
            id: 0,
            name: '',
            slotsRequired: 1,
            durationHours: 2,
            priority: 0,
        },
        onSubmitMeta: {
            action: 'edit' as EditAction,
        },
        onSubmit: async ({ value, meta, formApi }) => {
            switch (meta.action) {
                case 'edit': {
                    await updateExamAndRestrictions(
                        value.id,
                        {
                            name: value.name === '' ? null : value.name,
                            slotsRequired: value.slotsRequired,
                            durationHours: value.durationHours,
                            priority: value.priority,
                        },
                        restrictionIds,
                        restrictionMode,
                    )
                    break
                }
                case 'delete': {
                    if (
                        !confirm(
                            'Are you sure you want to delete this exam? This action cannot be undone.',
                        )
                    ) {
                        return
                    }
                    await db.delete(exam).where(eq(exam.id, value.id))
                    break
                }
            }

            await onSaved?.()
            onClose?.()
            restrictionIds.clear()
            restrictionMode = 'deny'
            formApi.reset()
        },
    }))

    /** AI-generated (GPT-5.3-codex). */
    async function getExamWithSubject(examId: number) {
        return db.query.exam.findFirst({
            where: { id: examId },
            with: { subject: true },
        })
    }

    /** AI-generated (GPT-5.3-codex). */
    async function loadExamTimeslotRestrictions(examId: number) {
        const selectedExam = await db.query.exam.findFirst({
            where: { id: examId },
            columns: { timeslotRestrictionMode: true },
            with: {
                restrictedTimeslots: { columns: { id: true } },
            },
        })
        if (!selectedExam)
            return

        restrictionMode = selectedExam.timeslotRestrictionMode ?? 'deny'
        restrictionIds.clear()
        for (const row of selectedExam.restrictedTimeslots) {
            restrictionIds.add(row.id)
        }
    }

    /** AI-generated (GPT-5.3-codex). */
    async function updateExamAndRestrictions(
        id: number,
        updates: Pick<
            typeof exam.$inferInsert,
            'slotsRequired' | 'durationHours' | 'priority' | 'name'
        >,
        selectedIds: Set<number>,
        mode: RestrictionMode,
    ) {
        await db.transaction(async (tx) => {
            await tx.update(exam).set(updates).where(eq(exam.id, id))
            await tx.delete(examTimeslotRestriction).where(eq(examTimeslotRestriction.examId, id))
            const nextMode = selectedIds.size > 0 ? mode : null
            await tx.update(exam).set({ timeslotRestrictionMode: nextMode }).where(eq(exam.id, id))
            if (selectedIds.size > 0) {
                await tx
                    .insert(examTimeslotRestriction)
                    .values([...selectedIds].map(timeslotId => ({ examId: id, timeslotId })))
            }
        })
    }

    /** AI-generated (GPT-5.3-codex). */
    function handleFormSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        editExamForm.handleSubmit()
    }

    $effect(() => {
        if (!(open && !previousOpen)) {
            previousOpen = open
            return
        }

        editExamForm.setFieldValue('id', examId)
        editExamForm.setFieldValue('durationHours', durationHours)
        editExamForm.setFieldValue('slotsRequired', slotsRequired)
        editExamForm.setFieldValue('priority', priority)
        editExamForm.setFieldValue('name', name === null ? '' : name)

        restrictionMode = 'deny'
        restrictionIds.clear()
        if (examId > 0) {
            void loadExamTimeslotRestrictions(examId)
        }

        previousOpen = open
    })
</script>

<editExamForm.Subscribe selector={state => ({ id: state.values.id })}>
    {#snippet children(state)}
        {@const examData = await getExamWithSubject(state.id)}
        <Dialog
            headline={`Editing ${examData?.subject?.name} Paper ${examData?.paper}`}
            bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
        >
            <form id='edit-exam-form' onsubmit={handleFormSubmit}>
                <editExamForm.Field name='name' validators={{ onChange: string() }}>
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label='Exam Name (Optional)'
                            type='text'
                            placeholder='Enter exam name'
                            helperText='Optional exam name. Exams are by default named after the paper number (e.g. Paper 1).'
                        />
                    {/snippet}
                </editExamForm.Field>

                <editExamForm.Field
                    name='durationHours'
                    validators={{ onChange: durationHoursValidator }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label='Duration (hours)'
                            type='number'
                            min='0.5'
                            step='0.25'
                            required
                            helperText='How long the exam lasts for (in hours)'
                        />
                    {/snippet}
                </editExamForm.Field>

                <editExamForm.Field
                    name='slotsRequired'
                    validators={{ onChange: slotsRequiredValidator }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label='Slots Required'
                            type='number'
                            min='1'
                            required
                            helperText='How many timeslots this exam runs over. For most exams, this will be 1'
                        />
                    {/snippet}
                </editExamForm.Field>

                <editExamForm.Field name='priority' validators={{ onChange: priorityValidator }}>
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label='Priority'
                            type='number'
                            min='0'
                            step='1'
                            required
                            helperText='Higher priority exams will be scheduled earlier when possible'
                        />
                    {/snippet}
                </editExamForm.Field>

                <fieldset>
                    <legend>Timeslot restrictions</legend>
                    <p>
                        Add exact slots this exam is allowed or denied to be scheduled in. If no
                        timeslots are allowed/denied, the exam can be scheduled in any slot. If
                        there are only denied slots, the exam can be scheduled in any slot except
                        the denied ones.
                    </p>
                    <TimeslotPicker
                        {allTimeslots}
                        selectedIds={restrictionIds}
                        mode={restrictionMode}
                    />
                </fieldset>
            </form>

            {#snippet buttons()}
                <Button variant='outlined' onclick={onClose}>Cancel</Button>
                <Button
                    variant='tonal'
                    color='error'
                    onclick={() => editExamForm.handleSubmit({ action: 'delete' })}>Delete</Button
                >
                <Button form='edit-exam-form' type='submit'>Save</Button>
            {/snippet}
        </Dialog>
    {/snippet}
</editExamForm.Subscribe>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    fieldset {
        border: 1px solid var(--m3c-outline-variant);
        border-radius: var(--m3-shape-medium);
        background-color: var(--m3c-surface);
        padding: 1rem 1.5rem 1.5rem;
        margin: 0;

        & legend {
            padding-inline: 0.25rem;
            @apply --m3-label-large;
        }

        & p {
            @apply --m3-body-medium;
            color: var(--m3c-on-surface-variant);
            margin-bottom: 1rem;
        }
    }
</style>
