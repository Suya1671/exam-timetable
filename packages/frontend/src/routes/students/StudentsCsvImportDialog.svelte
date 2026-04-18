<script lang='ts'>
    import { db } from '$lib/db'
    import { enrolledStudent, student, subject, subjectGrade } from '$lib/db/schema'
    import { EnhancedSelect, EnhancedTextInput } from '@exam-timetable/ui'
    import { createForm } from '@tanstack/svelte-form'
    import { eq, or } from 'drizzle-orm'
    import { Button, Dialog, Divider, ListItem, snackbar } from 'm3-svelte'
    import { SvelteMap, SvelteSet } from 'svelte/reactivity'
    import { integer, maxValue, minValue, number, pipe } from 'valibot'

    type ParsedCsvRow = {
        no: string
        surname: string
        name: string
        subjects: string[]
    }

    type CsvImportPreviewRow = ParsedCsvRow & {
        fullName: string
    }

    type CsvSubjectMapping = {
        csvSubjectName: string
        subjectId: string
    }

    type SelectOption = {
        value: string
        text: string
    }

    type CsvImportState = {
        fileName: string
        rows: CsvImportPreviewRow[]
        csvSubjectNames: string[]
        subjectOptions: SelectOption[]
        defaultSubjectOptions: SelectOption[]
    }

    const {
        open = false,
        onClose,
        onImported,
    }: {
        open?: boolean
        onClose?: () => void
        onImported?: () => void | Promise<void>
    } = $props()

    const MIN_GRADE = 1
    const MAX_GRADE = 12
    const DEFAULT_GRADE = 8
    const SKIP_SUBJECT_OPTION: SelectOption = { value: '0', text: '-- Skip --' }

    const gradeValidator = pipe(
        number('Grade must be a number.'),
        integer('Grade must be an integer.'),
        minValue(MIN_GRADE, `Grade must be at least ${MIN_GRADE}.`),
        maxValue(MAX_GRADE, `Grade must be at most ${MAX_GRADE}.`),
    )

    let csvImportState = $state<CsvImportState | null>(null)
    let csvImportInput = $state<HTMLInputElement | null>(null)
    let csvImporting = $state(false)
    let previousOpen = $state(false)

    const importCsvForm = createForm(() => ({
        defaultValues: {
            grade: DEFAULT_GRADE,
            defaultSubjectIds: [] as string[],
            mappings: [] as CsvSubjectMapping[],
        },
        onSubmit: async ({ value }) => {
            if (!csvImportState || csvImporting)
                return

            csvImporting = true
            try {
                const allowedForGrade = await subjectOptionsForGrade(value.grade)
                const allowedIds = new Set(
                    allowedForGrade
                        .map(option => Number(option.value))
                        .filter(id => Number.isFinite(id)),
                )

                const subjectIdByCsvName = new SvelteMap<string, number>()
                for (const mapping of value.mappings) {
                    const parsedId = Number(mapping.subjectId)
                    if (!Number.isFinite(parsedId) || parsedId <= 0)
                        continue
                    if (!allowedIds.has(parsedId))
                        continue
                    subjectIdByCsvName.set(mapping.csvSubjectName, parsedId)
                }

                const validDefaultSubjectIds = value.defaultSubjectIds
                    .map(id => Number(id))
                    .filter(id => Number.isFinite(id) && id > 0 && allowedIds.has(id))

                for (const row of csvImportState.rows) {
                    const studentId = await upsertStudentForImport(row, value.grade)
                    if (studentId === null)
                        continue

                    const finalSubjectIds = new SvelteSet<number>()

                    for (const csvSubjectName of row.subjects) {
                        const mappedSubjectId = subjectIdByCsvName.get(csvSubjectName)
                        if (mappedSubjectId)
                            finalSubjectIds.add(mappedSubjectId)
                    }

                    for (const defaultSubjectId of validDefaultSubjectIds) {
                        finalSubjectIds.add(defaultSubjectId)
                    }

                    for (const subjectId of finalSubjectIds) {
                        await db
                            .insert(enrolledStudent)
                            .values({ studentId, subjectId })
                            .onConflictDoNothing()
                    }
                }

                await onImported?.()
                closeDialog()
                snackbar('Import complete.')
            }
            finally {
                csvImporting = false
            }
        },
    }))

    /** AI-generated (GPT-5.2-codex). */
    function normalizeSubjectName(name: string) {
        return name.trim().replace(/\s+/g, ' ')
    }

    /** AI-generated (GPT-5.2-codex). */
    function parseSemicolonCsvLine(line: string) {
        return line.split(';').map(part => part.trim())
    }

    /** AI-generated (GPT-5.2-codex). */
    function parseStudentsCsv(csvText: string): ParsedCsvRow[] {
        const lines = csvText
            .split(/\r?\n/)
            .map(line => line.trimEnd())
            .filter(line => line.length > 0)

        const dataLines = lines.slice(2)

        return dataLines
            .map(parseSemicolonCsvLine)
            .map((cells) => {
                const no = cells[0] ?? ''
                const surname = cells[1] ?? ''
                const name = cells[2] ?? ''

                const subjects = cells
                    .slice(3, 9)
                    .map(subjectName => normalizeSubjectName(subjectName ?? ''))
                    .filter(subjectName => subjectName.length > 0)

                return { no, surname, name, subjects }
            })
            .filter(row => row.no.length > 0 && row.surname.length > 0 && row.name.length > 0)
    }

    /** AI-generated (GPT-5.2-codex). */
    function toGradeBoundedNumber(value: unknown) {
        const n = Number(value)
        if (!Number.isFinite(n))
            return DEFAULT_GRADE
        return Math.max(MIN_GRADE, Math.min(MAX_GRADE, n))
    }

    /** AI-generated (GPT-5.2-codex). */
    function toSubjectOptionsWithSkip(options: SelectOption[]) {
        return [SKIP_SUBJECT_OPTION, ...options]
    }

    /** AI-generated (GPT-5.2-codex). */
    async function subjectOptionsForGrade(grade: number): Promise<SelectOption[]> {
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

    /** AI-generated (GPT-5.2-codex). */
    async function findExistingStudentIdByCsvRow(row: CsvImportPreviewRow) {
        const studentName = row.fullName.trim()
        if (studentName.length === 0)
            return null

        const existing = await db
            .select({ id: student.id })
            .from(student)
            .where(
                or(
                    eq(student.name, studentName),
                    eq(student.name, `${row.surname} ${row.name}`.trim()),
                ),
            )
            .limit(1)

        return existing[0]?.id ?? null
    }

    /** AI-generated (GPT-5.2-codex). */
    async function upsertStudentForImport(row: CsvImportPreviewRow, grade: number) {
        const existingId = await findExistingStudentIdByCsvRow(row)

        if (existingId !== null) {
            await db.update(student).set({ grade }).where(eq(student.id, existingId))
            return existingId
        }

        const inserted = await db
            .insert(student)
            .values({
                name: row.fullName.trim(),
                grade,
            })
            .returning({ id: student.id })

        return inserted[0]?.id ?? null
    }

    /** AI-generated (GPT-5.2-codex). */
    async function buildCsvImportState(
        fileName: string,
        parsedRows: ParsedCsvRow[],
        grade: number,
    ): Promise<CsvImportState> {
        if (parsedRows.length === 0) {
            return {
                fileName,
                rows: [],
                csvSubjectNames: [],
                subjectOptions: [],
                defaultSubjectOptions: [],
            }
        }

        const gradeFilteredSubjectOptions = await subjectOptionsForGrade(grade)

        const rows: CsvImportPreviewRow[] = parsedRows.map(row => ({
            ...row,
            fullName: `${row.name} ${row.surname}`.trim(),
        }))

        const csvSubjectNames = [...new Set(parsedRows.flatMap(row => row.subjects))].sort(
            (a, b) => a.localeCompare(b),
        )

        return {
            fileName,
            rows,
            csvSubjectNames,
            subjectOptions: toSubjectOptionsWithSkip(gradeFilteredSubjectOptions),
            defaultSubjectOptions: gradeFilteredSubjectOptions,
        }
    }

    /** AI-generated (GPT-5.2-codex). */
    async function syncCsvImportOptionsForGrade(grade: number) {
        if (!csvImportState)
            return

        const gradeFilteredSubjectOptions = await subjectOptionsForGrade(grade)
        const nextSubjectOptions = toSubjectOptionsWithSkip(gradeFilteredSubjectOptions)

        csvImportState = {
            ...csvImportState,
            subjectOptions: nextSubjectOptions,
            defaultSubjectOptions: gradeFilteredSubjectOptions,
        }

        const allowed = new Set(nextSubjectOptions.map(option => option.value))

        const nextMappings = importCsvForm.state.values.mappings.map(mapping => ({
            ...mapping,
            subjectId: allowed.has(mapping.subjectId) ? mapping.subjectId : '0',
        }))
        importCsvForm.setFieldValue('mappings', nextMappings)

        const nextDefaults = importCsvForm.state.values.defaultSubjectIds.filter(id =>
            allowed.has(id),
        )
        importCsvForm.setFieldValue('defaultSubjectIds', nextDefaults)
    }

    /** AI-generated (GPT-5.2-codex). */
    async function handleCsvFilePicked(file: File | null) {
        if (!file)
            return

        const text = await file.text()
        const parsedRows = parseStudentsCsv(text)
        const grade = toGradeBoundedNumber(importCsvForm.state.values.grade)

        csvImportState = await buildCsvImportState(file.name, parsedRows, grade)

        if (parsedRows.length === 0) {
            importCsvForm.setFieldValue('mappings', [])
            snackbar('No student rows found in CSV.')
            return
        }

        const exactByNameInsensitive = new Map(
            (csvImportState.defaultSubjectOptions ?? []).map(s => [
                s.text.toLocaleLowerCase(),
                s.value,
            ]),
        )

        const initialMappings: CsvSubjectMapping[] = csvImportState.csvSubjectNames.map(
            csvSubjectName => ({
                csvSubjectName,
                subjectId: exactByNameInsensitive.get(csvSubjectName.toLocaleLowerCase()) ?? '0',
            }),
        )

        importCsvForm.setFieldValue('mappings', initialMappings)
    }

    /** AI-generated (GPT-5.2-codex). */
    function triggerCsvPicker() {
        csvImportInput?.click()
    }

    /** AI-generated (GPT-5.2-codex). */
    function closeDialog() {
        onClose?.()
    }

    /** AI-generated (GPT-5.2-codex). */
    function resetImportDialogState() {
        csvImportState = null
        importCsvForm.setFieldValue('grade', DEFAULT_GRADE)
        importCsvForm.setFieldValue('defaultSubjectIds', [])
        importCsvForm.setFieldValue('mappings', [])
    }

    /** AI-generated (GPT-5.2-codex). */
    function handleFormSubmit(e: SubmitEvent, form: { handleSubmit: () => void }) {
        e.preventDefault()
        e.stopPropagation()
        form.handleSubmit()
    }

    $effect(() => {
        if (open && !previousOpen) {
            resetImportDialogState()
        }

        previousOpen = open
    })
</script>

<div class='import-dialog-shell'>
    <Dialog
        headline='Import Students from CSV'
        bind:open={() => open, (nextOpen: boolean) => !nextOpen && closeDialog()}
    >
        <form id='import-csv-form' onsubmit={e => handleFormSubmit(e, importCsvForm)}>
            <input
                bind:this={csvImportInput}
                type='file'
                accept='.csv,text/csv'
                class='import-file-input'
                onchange={async (ev) => {
                    const target = ev.currentTarget as HTMLInputElement
                    const file = target.files?.[0] ?? null
                    await handleCsvFilePicked(file)
                    target.value = ''
                }}
            />

            <fieldset>
                <legend>CSV file</legend>
                <p>Pick a subject-choice CSV to preview and import students.</p>

                <div class='import-file-row'>
                    <Button variant='outlined' type='button' onclick={triggerCsvPicker}
                    >Choose CSV File</Button
                    >
                    {#if csvImportState}
                        <span>{csvImportState.fileName}</span>
                    {/if}
                </div>

                {#if csvImportState}
                    <p>Rows parsed: <strong>{csvImportState.rows.length}</strong></p>
                {/if}
            </fieldset>

            {#if csvImportState}
                <fieldset>
                    <legend>Import grade</legend>
                    <p>All imported students will be set to this grade.</p>

                    <importCsvForm.Field
                        name='grade'
                        validators={{ onChange: gradeValidator }}
                        listeners={{
                            onChange: async ({ value }) => {
                                await syncCsvImportOptionsForGrade(toGradeBoundedNumber(value))
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
                    </importCsvForm.Field>
                </fieldset>

                <fieldset>
                    <legend>Default subjects</legend>
                    <p>These subjects are added to every imported student.</p>

                    <importCsvForm.Field name='defaultSubjectIds'>
                        {#snippet children(field)}
                            <ul class='default-subject-list'>
                                {#each csvImportState?.defaultSubjectOptions ?? [] as option (option.value)}
                                    <li>
                                        <label class='default-subject-item'>
                                            <input
                                                type='checkbox'
                                                checked={field.state.value.includes(option.value)}
                                                onchange={(ev) => {
                                                    const checked = (
                                                        ev.currentTarget as HTMLInputElement
                                                    ).checked
                                                    const nextValues = checked
                                                        ? [...field.state.value, option.value]
                                                        : field.state.value.filter(
                                                            v => v !== option.value,
                                                        )
                                                    field.handleChange(nextValues)
                                                }}
                                            />
                                            <span>{option.text}</span>
                                        </label>
                                    </li>
                                {/each}
                            </ul>
                        {/snippet}
                    </importCsvForm.Field>
                </fieldset>

                <fieldset>
                    <legend>Subject mapping</legend>
                    <p>
                        Map each subject in the CSV file to a subject in the system. Use Skip to
                        ignore a subject.
                    </p>

                    <ul class='mapping-list'>
                        {#each csvImportState.csvSubjectNames as csvSubjectName, index (csvSubjectName)}
                            <ListItem headline={csvSubjectName}>
                                {#snippet trailing()}
                                    <div class='mapping-trailing'>
                                        <importCsvForm.Field name={`mappings[${index}].subjectId`}>
                                            {#snippet children(field)}
                                                <EnhancedSelect
                                                    {field}
                                                    label='Map to Subject'
                                                    variant='outlined'
                                                    options={csvImportState?.subjectOptions ?? []}
                                                />
                                            {/snippet}
                                        </importCsvForm.Field>

                                        <importCsvForm.Field
                                            name={`mappings[${index}].csvSubjectName`}
                                        >
                                            {#snippet children(field)}
                                                <input
                                                    type='hidden'
                                                    name={field.name}
                                                    value={csvSubjectName}
                                                />
                                            {/snippet}
                                        </importCsvForm.Field>
                                    </div>
                                {/snippet}
                            </ListItem>
                            <Divider />
                        {/each}
                    </ul>
                </fieldset>

                <fieldset>
                    <legend>Preview</legend>
                    <table>
                        <thead>
                            <tr>
                                <th>No</th>
                                <th>Name</th>
                                <th>Subjects in CSV</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each csvImportState.rows as row (`${row.no}-${row.fullName}`)}
                                <tr>
                                    <td>{row.no}</td>
                                    <td>{row.fullName}</td>
                                    <td>{row.subjects.join(', ')}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </fieldset>
            {/if}
        </form>

        {#snippet buttons()}
            <Button variant='outlined' onclick={closeDialog} disabled={csvImporting}>Cancel</Button>
            <Button
                form='import-csv-form'
                type='submit'
                disabled={!csvImportState || csvImportState.rows.length === 0 || csvImporting}
            >
                {csvImporting ? 'Importing...' : 'Import'}
            </Button>
        {/snippet}
    </Dialog>
</div>

<style>
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    fieldset {
        margin: 0;
        padding: 0;
        border: none;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;

        & legend {
            font-weight: 600;
            margin-bottom: 0.25rem;
        }

        & p {
            margin: 0;
            opacity: 0.8;
        }
    }

    .import-file-input {
        display: none;
    }

    .import-file-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .mapping-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .default-subject-list {
        margin: 0;
        padding: 0;
        list-style: none;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
        gap: 0.5rem 0.75rem;
    }

    .default-subject-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .mapping-trailing {
        min-width: min(360px, 55vw);
    }

    table {
        width: 100%;
        border-collapse: collapse;

        & th,
        & td {
            text-align: left;
            padding: 0.5rem 0.625rem;
            border-bottom: 1px solid rgba(0, 0, 0, 0.08);
            vertical-align: top;
        }

        & thead th {
            position: sticky;
            top: 0;
            background: #fff;
            z-index: 1;
        }
    }

    .import-dialog-shell > :global(dialog.m3-container) {
        max-width: min(1100px, 96vw);
        width: min(1100px, 96vw);
        max-height: 92vh;
    }
</style>
