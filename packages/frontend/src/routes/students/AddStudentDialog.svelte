<script lang='ts'>
    import { db } from '$lib/db'
    import { enrolledStudent, student, subject, subjectGrade } from '$lib/db/schema'
    import EnhancedTextInput from '$lib/EnhancedTextInput.svelte'
    import { createForm } from '@tanstack/svelte-form'
    import { eq } from 'drizzle-orm'
    import { Button, Chip, Dialog } from 'm3-svelte'
    import {
        DEFAULT_GRADE,
        gradeValidator,
        MAX_GRADE,
        MIN_GRADE,
        studentNameValidator,
    } from './forms'

    type SelectOption = {
        value: string
        text: string
    }

    const {
        open = false,
        onClose,
        onSaved,
    }: {
        open?: boolean
        onClose?: () => void
        onSaved?: () => void | Promise<void>
    } = $props()

    let previousOpen = $state(false)
    let gradeSubjectOptions = $state<SelectOption[]>([])

    const addStudentForm = createForm(() => ({
        defaultValues: {
            grade: DEFAULT_GRADE,
            batchNames: '',
            subjectIds: [] as string[],
        },
        onSubmit: async ({ value, formApi }) => {
            const batchNames = parseStudentNames(value.batchNames)
            if (batchNames.length === 0)
                return

            const allowedSubjectOptions = await getSubjectOptionsForGrade(value.grade)
            const allowedSubjectIds = new Set(
                allowedSubjectOptions
                    .map(option => Number(option.value))
                    .filter(id => Number.isFinite(id) && id > 0),
            )
            const selectedSubjectIds = [...new Set(value.subjectIds)]
                .map(id => Number(id))
                .filter(id => Number.isFinite(id) && id > 0 && allowedSubjectIds.has(id))

            await db.transaction(async (tx) => {
                for (const batchName of batchNames) {
                    const [createdStudent] = await tx
                        .insert(student)
                        .values({
                            name: batchName,
                            grade: value.grade,
                        })
                        .returning({ id: student.id })

                    const createdStudentId = createdStudent?.id
                    if (!createdStudentId)
                        continue

                    for (const selectedSubjectId of selectedSubjectIds) {
                        await tx
                            .insert(enrolledStudent)
                            .values({ studentId: createdStudentId, subjectId: selectedSubjectId })
                            .onConflictDoNothing()
                    }
                }
            })

            await onSaved?.()
            onClose?.()
            formApi.reset()
        },
    }))

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function clampGradeToRange(value: unknown) {
        const n = Number(value)
        if (!Number.isFinite(n))
            return DEFAULT_GRADE
        return Math.max(MIN_GRADE, Math.min(MAX_GRADE, n))
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function parseStudentNames(rawValue: string) {
        const unique = new Set<string>()

        for (const part of rawValue.split(/[\n,;]+/)) {
            const normalized = part.trim().replace(/\s+/g, ' ')
            if (normalized.length === 0)
                continue
            unique.add(normalized)
        }

        return [...unique]
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    async function getSubjectOptionsForGrade(grade: number): Promise<SelectOption[]> {
        if (!Number.isFinite(grade) || grade < MIN_GRADE || grade > MAX_GRADE)
            return []

        const rows = await db
            .select({ value: subject.id, text: subject.name })
            .from(subjectGrade)
            .innerJoin(subject, eq(subject.id, subjectGrade.subjectId))
            .where(eq(subjectGrade.grade, grade))
            .orderBy(subject.name)

        return rows.map(row => ({ value: row.value.toString(), text: row.text }))
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    async function syncSubjectOptionsForGrade(gradeValue: unknown) {
        const grade = clampGradeToRange(gradeValue)
        const nextOptions = await getSubjectOptionsForGrade(grade)
        gradeSubjectOptions = nextOptions

        const allowedValues = new Set(nextOptions.map(option => option.value))
        const nextSelected = addStudentForm.state.values.subjectIds.filter(id =>
            allowedValues.has(id),
        )
        addStudentForm.setFieldValue('subjectIds', nextSelected)
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function toggleSubjectSelection(
        field: { state: { value: string[] }, handleChange: (v: string[]) => void },
        id: string,
    ) {
        const hasId = field.state.value.includes(id)
        const nextValues = hasId
            ? field.state.value.filter(value => value !== id)
            : [...field.state.value, id]
        field.handleChange(nextValues)
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function setAllSubjectSelections(
        field: { handleChange: (v: string[]) => void },
        allSelected: boolean,
    ) {
        field.handleChange(allSelected ? gradeSubjectOptions.map(option => option.value) : [])
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    async function seedFormValues() {
        addStudentForm.setFieldValue('grade', DEFAULT_GRADE)
        addStudentForm.setFieldValue('batchNames', '')
        addStudentForm.setFieldValue('subjectIds', [])
        await syncSubjectOptionsForGrade(DEFAULT_GRADE)
    }

    /** AI-generated (gpt-5.3-codex/OpenCode). */
    function handleFormSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        addStudentForm.handleSubmit()
    }

    $effect(() => {
        if (open && !previousOpen) {
            void seedFormValues()
        }

        previousOpen = open
    })
</script>

<Dialog
    headline='Add Students'
    bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
    <form id='add-student-form' onsubmit={handleFormSubmit}>
        <addStudentForm.Field name='batchNames' validators={{ onChange: studentNameValidator }}>
            {#snippet children(field)}
                {@const studentCount = parseStudentNames(field.state.value).length}
                <div class='batch-name-field'>
                    <label for='batchNamesInput'>Students</label>
                    <textarea
                        id='batchNamesInput'
                        name={field.name}
                        required
                        rows='8'
                        placeholder='One student per line'
                        value={field.state.value}
                        oninput={event =>
                            field.handleChange((event.currentTarget as HTMLTextAreaElement).value)}
                        onblur={() => field.handleBlur()}
                    ></textarea>
                    <p>One student per line. Commas and semicolons also work.</p>
                    <p>Students to add: <strong>{studentCount}</strong></p>
                </div>
            {/snippet}
        </addStudentForm.Field>

        <addStudentForm.Field
            name='grade'
            validators={{ onChange: gradeValidator }}
            listeners={{
                onChange: async ({ value }) => {
                    await syncSubjectOptionsForGrade(value)
                },
            }}
        >
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
        </addStudentForm.Field>

        <addStudentForm.Field name='subjectIds'>
            {#snippet children(field)}
                {@const allSelected
                    = gradeSubjectOptions.length > 0
                        && field.state.value.length === gradeSubjectOptions.length}
                <fieldset class='subject-selector'>
                    <legend>Subjects for all new students</legend>
                    <div class='subject-actions'>
                        <Chip
                            variant='assist'
                            disabled={gradeSubjectOptions.length === 0 || allSelected}
                            onclick={() => setAllSubjectSelections(field, true)}
                        >
                            Select all
                        </Chip>
                        <Chip
                            variant='assist'
                            disabled={field.state.value.length === 0}
                            onclick={() => setAllSubjectSelections(field, false)}
                        >
                            Clear
                        </Chip>
                    </div>

                    {#if gradeSubjectOptions.length === 0}
                        <p>No subjects are available for this grade.</p>
                    {:else}
                        <div class='subject-chip-list' role='group' aria-label='Subject selection'>
                            {#each gradeSubjectOptions as option (option.value)}
                                <Chip
                                    variant='input'
                                    selected={field.state.value.includes(option.value)}
                                    onclick={() => toggleSubjectSelection(field, option.value)}
                                >
                                    {option.text}
                                </Chip>
                            {/each}
                        </div>
                    {/if}
                </fieldset>
            {/snippet}
        </addStudentForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant='outlined' onclick={onClose}>Cancel</Button>
        <Button form='add-student-form' type='submit'>Add students</Button>
    {/snippet}
</Dialog>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .batch-name-field {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;

        & label {
            font-weight: 600;
        }

        & textarea {
            width: 100%;
            resize: vertical;
            min-height: 9rem;
            padding: 0.75rem;
            border-radius: 0.75rem;
            border: 1px solid var(--m3c-outline-variant);
            background: var(--m3c-surface);
            color: var(--m3c-on-surface);
            font: inherit;
        }

        & p {
            margin: 0;
            opacity: 0.8;
            font-size: 0.875rem;
        }
    }

    .subject-selector {
        border: 1px solid var(--m3c-outline-variant);
        border-radius: 0.75rem;
        padding: 0.75rem;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;

        & legend {
            padding-inline: 0.25rem;
            font-weight: 600;
        }

        & p {
            margin: 0;
            opacity: 0.8;
        }
    }

    .subject-actions {
        display: flex;
        gap: 0.5rem;
        flex-wrap: wrap;
    }

    .subject-chip-list {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }
</style>
