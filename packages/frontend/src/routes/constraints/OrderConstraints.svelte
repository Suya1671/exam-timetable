<script lang='ts'>
    import { db } from '$lib/db'
    import { exam, examOrderConstraint, subject } from '$lib/db/schema'
    import { formatExamLabel } from '$lib/examDisplay'
    import { EnhancedSelect } from '@exam-timetable/ui'
    import { createForm } from '@tanstack/svelte-form'
    import { and, asc, eq } from 'drizzle-orm'
    import { alias } from 'drizzle-orm/sqlite-core'
    import { Button, Card } from 'm3-svelte'
    import { nonEmpty, pipe, string } from 'valibot'

    type ConstraintWithExams = {
        exam1Id: number
        exam2Id: number
        firstExam: { id: number, grade: number, paper: number, subject: { name: string } }
        secondExam: { id: number, grade: number, paper: number, subject: { name: string } }
    }

    const firstExam = alias(exam, 'firstExam')
    const secondExam = alias(exam, 'secondExam')
    const firstSubject = alias(subject, 'firstSubject')
    const secondSubject = alias(subject, 'secondSubject')

    async function loadConstraints(): Promise<ConstraintWithExams[]> {
        const rows = await db
            .select()
            .from(examOrderConstraint)
            .innerJoin(firstExam, eq(examOrderConstraint.exam1Id, firstExam.id))
            .innerJoin(firstSubject, eq(firstExam.subjectId, firstSubject.id))
            .innerJoin(secondExam, eq(examOrderConstraint.exam2Id, secondExam.id))
            .innerJoin(secondSubject, eq(secondExam.subjectId, secondSubject.id))
            .orderBy(asc(firstExam.grade), asc(firstSubject.name), asc(firstExam.paper))

        return rows.map(row => ({
            exam1Id: row.exam_order_constraint.exam1Id,
            exam2Id: row.exam_order_constraint.exam2Id,
            firstExam: { ...row.firstExam, subject: row.firstSubject },
            secondExam: { ...row.secondExam, subject: row.secondSubject },
        }))
    }

    async function getSubjectOptions() {
        return (await db.query.subject.findMany({ orderBy: { name: 'asc' } })).map(s => ({
            value: s.id.toString(),
            text: s.name,
        }))
    }

    async function getGradeOptions(subjectId: string) {
        const parsed = Number(subjectId)
        if (!Number.isFinite(parsed))
            return []
        const grades = await db.query.subjectGrade.findMany({
            where: { subjectId: parsed },
            orderBy: { grade: 'asc' },
        })
        return grades.map(g => ({ value: g.grade.toString(), text: `Grade ${g.grade}` }))
    }

    async function getPaperOptions(subjectId: string, grade: string) {
        const parsedSubject = Number(subjectId)
        const parsedGrade = Number(grade)
        if (!Number.isFinite(parsedSubject) || !parsedGrade)
            return []
        const papers = await db.query.exam.findMany({
            where: { subjectId: parsedSubject, grade: parsedGrade },
            with: { subject: true },
            orderBy: { paper: 'asc' },
        })
        return papers.map(p => ({ value: p.id.toString(), text: formatExamLabel(p) }))
    }

    let constraints = $state(loadConstraints())

    const form = createForm(() => ({
        defaultValues: {
            grade1: '',
            subject1Id: '',
            exam1Id: '',
            grade2: '',
            subject2Id: '',
            exam2Id: '',
        },
        onSubmit: async ({ value, formApi }) => {
            const exam1Id = Number(value.exam1Id)
            const exam2Id = Number(value.exam2Id)
            if (!Number.isFinite(exam1Id) || !Number.isFinite(exam2Id) || exam1Id === exam2Id)
                return

            await db
                .insert(examOrderConstraint)
                .values({
                    exam1Id,
                    exam2Id,
                })
                .onConflictDoNothing()

            constraints = loadConstraints()
            formApi.reset()
        },
    }))

    function handleSubmit(e: SubmitEvent) {
        e.preventDefault()
        e.stopPropagation()
        form.handleSubmit()
    }

    async function remove(exam1Id: number, exam2Id: number) {
        if (!confirm('Delete this constraint?'))
            return
        await db
            .delete(examOrderConstraint)
            .where(
                and(
                    eq(examOrderConstraint.exam1Id, exam1Id),
                    eq(examOrderConstraint.exam2Id, exam2Id),
                ),
            )
        constraints = loadConstraints()
    }
</script>

<Card variant='filled'>
    <section>
        <header>
            <h2>Order Constraints</h2>
            <p>Exams that must be scheduled before other exams.</p>
        </header>

        <form id='order-constraint-form' onsubmit={handleSubmit}>
            <div class='exams'>
                <fieldset>
                    <legend>First Exam (Before)</legend>
                    <form.Field
                        name='subject1Id'
                        validators={{ onChange: pipe(string(), nonEmpty('Pick subject')) }}
                    >
                        {#snippet children(field)}
                            {@const opts = await getSubjectOptions()}
                            <EnhancedSelect
                                {field}
                                variant='outlined'
                                label='Subject'
                                options={opts}
                                onValueChange={() => {
                                    form.setFieldValue('grade1', '')
                                    form.setFieldValue('exam1Id', '')
                                }}
                            />
                        {/snippet}
                    </form.Field>
                    <form.Subscribe selector={s => s.values.subject1Id}>
                        {#snippet children(sid)}
                            <form.Field
                                name='grade1'
                                validators={{ onChange: pipe(string(), nonEmpty('Pick grade')) }}
                            >
                                {#snippet children(field)}
                                    {@const opts = await getGradeOptions(sid)}
                                    <EnhancedSelect
                                        {field}
                                        variant='outlined'
                                        label='Grade'
                                        options={opts}
                                        onValueChange={() => form.setFieldValue('exam1Id', '')}
                                        disabled={!sid}
                                    />
                                {/snippet}
                            </form.Field>
                        {/snippet}
                    </form.Subscribe>
                    <form.Subscribe
                        selector={s => ({ g: s.values.grade1, s: s.values.subject1Id })}
                    >
                        {#snippet children(state)}
                            <form.Field
                                name='exam1Id'
                                validators={{ onChange: pipe(string(), nonEmpty('Pick paper')) }}
                            >
                                {#snippet children(field)}
                                    {@const opts = await getPaperOptions(state.s, state.g)}
                                    <EnhancedSelect
                                        {field}
                                        variant='outlined'
                                        label='Paper'
                                        options={opts}
                                        disabled={!state.g || !state.s}
                                    />
                                {/snippet}
                            </form.Field>
                        {/snippet}
                    </form.Subscribe>
                </fieldset>

                <fieldset>
                    <legend>Second Exam (After)</legend>
                    <form.Field
                        name='subject2Id'
                        validators={{ onChange: pipe(string(), nonEmpty('Pick subject')) }}
                    >
                        {#snippet children(field)}
                            {@const opts = await getSubjectOptions()}
                            <EnhancedSelect
                                {field}
                                variant='outlined'
                                label='Subject'
                                options={opts}
                                onValueChange={() => {
                                    form.setFieldValue('grade2', '')
                                    form.setFieldValue('exam2Id', '')
                                }}
                            />
                        {/snippet}
                    </form.Field>
                    <form.Subscribe selector={s => s.values.subject2Id}>
                        {#snippet children(sid)}
                            <form.Field
                                name='grade2'
                                validators={{ onChange: pipe(string(), nonEmpty('Pick grade')) }}
                            >
                                {#snippet children(field)}
                                    {@const opts = await getGradeOptions(sid)}
                                    <EnhancedSelect
                                        {field}
                                        variant='outlined'
                                        label='Grade'
                                        options={opts}
                                        onValueChange={() => form.setFieldValue('exam2Id', '')}
                                        disabled={!sid}
                                    />
                                {/snippet}
                            </form.Field>
                        {/snippet}
                    </form.Subscribe>
                    <form.Subscribe
                        selector={s => ({ g: s.values.grade2, s: s.values.subject2Id })}
                    >
                        {#snippet children(state)}
                            <form.Field
                                name='exam2Id'
                                validators={{ onChange: pipe(string(), nonEmpty('Pick paper')) }}
                            >
                                {#snippet children(field)}
                                    {@const opts = await getPaperOptions(state.s, state.g)}
                                    <EnhancedSelect
                                        {field}
                                        variant='outlined'
                                        label='Paper'
                                        options={opts}
                                        disabled={!state.g || !state.s}
                                    />
                                {/snippet}
                            </form.Field>
                        {/snippet}
                    </form.Subscribe>
                </fieldset>
            </div>

            <footer>
                <form.Subscribe selector={s => ({ can: s.canSubmit, pristine: s.isPristine })}>
                    {#snippet children(s)}
                        <Button
                            form='order-constraint-form'
                            type='submit'
                            variant='filled'
                            disabled={!s.can || s.pristine}>Add</Button
                        >
                    {/snippet}
                </form.Subscribe>
            </footer>
        </form>

        {#await constraints then loaded}
            {#if loaded.length === 0}
                <p class='empty'>No order constraints yet.</p>
            {:else}
                <table>
                    <thead><tr><th>Exam Order</th><th></th></tr></thead>
                    <tbody>
                        {#each loaded as c (`${c.exam1Id}-${c.exam2Id}`)}
                            <tr>
                                <td
                                >{formatExamLabel(c.firstExam)} must be before {formatExamLabel(
                                    c.secondExam,
                                )}</td
                                >
                                <td
                                ><Button
                                    variant='text'
                                    color='error'
                                    onclick={() => remove(c.exam1Id, c.exam2Id)}>Delete</Button
                                ></td
                                >
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        {/await}
    </section>
</Card>

<style>
    section {
        padding: 1rem;
    }

    header {
        margin-bottom: 1rem;

        h2 {
            @apply --m3-title-large;
            color: var(--m3c-on-surface);
        }

        p {
            @apply --m3-body-medium;
            color: var(--m3c-on-surface-variant);
        }
    }

    .exams {
        display: flex;
        gap: 1rem;

        @media (max-width: 600px) {
            flex-direction: column;
        }
    }

    /* TODO: get this looking less ugly as fu- */
    fieldset {
        flex: 1;
        display: grid;
        gap: 0.75rem;
        padding: 1rem;
        border: 1px solid var(--m3c-outline-variant);
        border-radius: var(--m3-shape-medium);
        background-color: var(--m3c-surface);

        legend {
            @apply --m3-title-small;
            color: var(--m3c-on-surface-variant);
            background-color: var(--m3c-surface-container-highest);
            border-radius: var(--m3-shape-small);
            padding-inline: 0.25rem;
        }

        :global(label) {
            border-radius: var(--m3-shape-small);
        }
    }
    footer {
        margin-top: 1rem;
    }

    .empty {
        @apply --m3-body-medium;
        color: var(--m3c-on-surface-variant);
        margin-top: 1rem;
    }

    table {
        width: 100%;
        margin-top: 1rem;
        border-collapse: collapse;
        border-radius: var(--m3-shape-medium);
        overflow: hidden;
        background-color: var(--m3c-surface-container-low, var(--m3c-surface));
        outline: 1px solid var(--m3c-outline-variant);

        & thead tr {
            background-color: var(--m3c-surface-container-highest);
            border-bottom: 1px solid var(--m3c-outline-variant);
        }

        & th {
            @apply --m3-label-large;
            color: var(--m3c-on-surface-variant);
            text-align: left;
            padding: 0.875rem 1rem;
            white-space: nowrap;
            user-select: none;
        }

        & tbody tr {
            border-bottom: 1px solid var(--m3c-outline-variant);
            transition: background-color var(--m3-easing-fast, 150ms ease);

            &:last-child {
                border-bottom: none;
            }

            &:hover {
                background-color: color-mix(in oklch, var(--m3c-on-surface) 8%, transparent);
            }
        }

        & td {
            @apply --m3-body-medium;
            color: var(--m3c-on-surface);
            padding: 0.75rem 1rem;
            vertical-align: middle;
        }
    }
</style>
