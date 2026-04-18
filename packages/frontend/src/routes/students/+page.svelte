<script lang='ts'>
    import type { IApi, IColumnConfig, IRow } from '@svar-ui/svelte-grid'
    import type {
        StudentsActiveDialog as ActiveDialog,
        AddAllSubjectsEvent,
        AddSubjectEvent,
        ClearSubjectsEvent,
        DeleteStudentEvent,
        EditStudentEvent,
        RemoveSubjectEvent,
    } from './types'
    import { db } from '$lib/db'
    import { enrolledStudent, student, subject, subjectGrade } from '$lib/db/schema'
    import { Grid, Willow } from '@svar-ui/svelte-grid'
    import { and, eq, isNull } from 'drizzle-orm'
    import { Button, Select, Snackbar, snackbar, TextField } from 'm3-svelte'
    import AddStudentDialog from './AddStudentDialog.svelte'
    import AddStudentSubjectDialog from './AddStudentSubjectDialog.svelte'
    import EditStudentDialog from './EditStudentDialog.svelte'
    import StudentActionsCell from './StudentActionsCell.svelte'
    import StudentsCsvImportDialog from './StudentsCsvImportDialog.svelte'
    import SubjectChipsCell from './SubjectChipsCell.svelte'

    function closeDialog() {
        activeDialog = null
    }

    /** AI-generated (GPT-5.3-codex). */
    async function getStudents() {
        return await db.query.student.findMany({
            with: {
                subjects: {
                    orderBy: {
                        name: 'asc',
                    },
                },
            },
            orderBy: { grade: 'asc', name: 'asc' },
        })
    }

    /** AI-generated (GPT-5.3-codex). */
    async function getGradeOptions() {
        const grades = await db.selectDistinct({ grade: student.grade }).from(student)
        const mappedGrades = grades.map(({ grade }) => ({
            text: `Grade ${grade}`,
            value: grade.toString(),
        }))
        return [...mappedGrades, { text: 'All', value: 'all' }]
    }

    /** AI-generated (GPT-5.3-codex). */
    function getAvailableSubjectsForStudentQuery(studentId: number) {
        return db
            .select({ id: subject.id, name: subject.name })
            .from(subject)
            .innerJoin(
                subjectGrade,
                and(eq(subjectGrade.subjectId, subject.id), eq(subjectGrade.grade, student.grade)),
            )
            .innerJoin(student, eq(student.id, studentId))
            .leftJoin(
                enrolledStudent,
                and(
                    eq(enrolledStudent.subjectId, subject.id),
                    eq(enrolledStudent.studentId, student.id),
                ),
            )
            .where(isNull(enrolledStudent.studentId))
    }

    let students = $state(await getStudents())
    let allGradeOptions = $state(await getGradeOptions())

    let gridApi = $state<IApi | null>(null)
    let studentNameFilter = $state('')
    let gradeFilter = $state<null | number>(null)

    let activeDialog = $state<ActiveDialog>(null)

    function openAddStudentDialog() {
        activeDialog = { type: 'addStudent' }
    }

    function openEditStudentDialog(studentId: number, name: string, grade: number) {
        activeDialog = { type: 'editStudent', studentId, name, grade }
    }

    function openAddSubjectDialog(studentId: number) {
        activeDialog = { type: 'addSubject', studentId }
    }

    function openImportCsvDialog() {
        activeDialog = { type: 'importCsv' }
    }

    async function refreshData() {
        students = await getStudents()
        allGradeOptions = await getGradeOptions()
    }

    function initGrid(api: IApi) {
        gridApi = api
    }

    const gridFilter = $derived.by(() => {
        const normalizedName = studentNameFilter.trim().toLowerCase()

        return (row: IRow) => {
            const matchesName
                = normalizedName.length === 0
                    || (typeof row.name === 'string' && row.name.toLowerCase().includes(normalizedName))

            const matchesGrade
                = gradeFilter === null
                    || (typeof row.grade === 'number' && row.grade === gradeFilter)

            return matchesName && matchesGrade
        }
    })

    $effect(() => {
        if (!gridApi)
            return
        gridApi.exec('filter-rows', { filter: gridFilter })
    })

    const columns: IColumnConfig[] = [
        { id: 'name', header: 'Name', width: 180 },
        { id: 'grade', header: 'Grade', width: 120 },
        {
            id: 'subjects',
            header: 'Subjects',
            cell: SubjectChipsCell,
            width: 280,
            flexgrow: 1,
        },
        {
            id: 'actions',
            header: 'Actions',
            cell: StudentActionsCell,
            width: 160,
        },
    ]

    async function handleAddSubjectEvent(ev: AddSubjectEvent) {
        const availableSubjects = await getAvailableSubjectsForStudentQuery(ev.studentId)

        if (availableSubjects.length === 0) {
            snackbar('No available subjects to add.')
            return
        }

        openAddSubjectDialog(ev.studentId)
    }

    async function handleRemoveSubjectEvent(ev: RemoveSubjectEvent) {
        if (!confirm(`Remove "${ev.subjectName}" from this student?`))
            return

        await db
            .delete(enrolledStudent)
            .where(
                and(
                    eq(enrolledStudent.studentId, ev.studentId),
                    eq(enrolledStudent.subjectId, ev.subjectId),
                ),
            )
        await refreshData()
    }

    function handleEditStudentEvent(ev: EditStudentEvent) {
        openEditStudentDialog(
            Number(ev.studentId),
            String(ev.studentName ?? ''),
            Number(ev.studentGrade),
        )
    }

    async function handleDeleteStudentEvent(ev: DeleteStudentEvent) {
        if (!confirm(`Delete "${ev.studentName}"?`))
            return

        await db.delete(student).where(eq(student.id, ev.studentId))
        await refreshData()
    }

    async function handleAddAllSubjectsEvent(ev: AddAllSubjectsEvent) {
        await db.insert(enrolledStudent).select(
            db
                .select({
                    studentId: student.id,
                    subjectId: subject.id,
                })
                .from(subject)
                .innerJoin(
                    subjectGrade,
                    and(
                        eq(subjectGrade.subjectId, subject.id),
                        eq(subjectGrade.grade, student.grade),
                    ),
                )
                .innerJoin(student, eq(student.id, ev.studentId))
                .leftJoin(
                    enrolledStudent,
                    and(
                        eq(enrolledStudent.subjectId, subject.id),
                        eq(enrolledStudent.studentId, student.id),
                    ),
                )
                .where(isNull(enrolledStudent.studentId)),
        )
        await refreshData()
    }

    async function handleClearSubjectsEvent(ev: ClearSubjectsEvent) {
        if (!confirm(`Clear all subjects from "${ev.studentName}"?`))
            return

        await db.delete(enrolledStudent).where(eq(enrolledStudent.studentId, ev.studentId))
        await refreshData()
    }
</script>

<header>
    <h1>Students</h1>

    <menu aria-label='Student actions'>
        <li><Button iconType='left' onclick={openAddStudentDialog}>Add Students</Button></li>
        <li><Button onclick={openImportCsvDialog}>Import from CSV</Button></li>
    </menu>
</header>

<main>
    <form aria-label='Filter students' onsubmit={event => event.preventDefault()}>
        <TextField
            label='Student name'
            placeholder='Search by student name'
            bind:value={studentNameFilter}
        ></TextField>

        <Select
            label='Grade'
            placeholder='Filter by grade'
            bind:value={() => (gradeFilter === null ? 'all' : gradeFilter.toString()),
                v => (gradeFilter = v === 'all' ? null : Number(v))}
            options={allGradeOptions}
        ></Select>
    </form>

    <div class='students-grid'>
        <Willow>
            <Grid
                data={students}
                {columns}
                init={initGrid}
                filterValues={{}}
                autoRowHeight
                sizes={{ rowHeight: 56 }}
                onaddsubject={ev => handleAddSubjectEvent(ev)}
                onremovesubject={ev => handleRemoveSubjectEvent(ev)}
                oneditstudent={ev => handleEditStudentEvent(ev)}
                ondeletestudent={ev => handleDeleteStudentEvent(ev)}
                onaddallsubjects={ev => handleAddAllSubjectsEvent(ev)}
                onclearsubjects={ev => handleClearSubjectsEvent(ev)}
            ></Grid>
        </Willow>
    </div>
</main>

<AddStudentDialog
    open={activeDialog?.type === 'addStudent'}
    onClose={closeDialog}
    onSaved={refreshData}
/>

<EditStudentDialog
    open={activeDialog?.type === 'editStudent'}
    studentId={activeDialog?.type === 'editStudent' ? activeDialog.studentId : 0}
    name={activeDialog?.type === 'editStudent' ? activeDialog.name : ''}
    grade={activeDialog?.type === 'editStudent' ? activeDialog.grade : 0}
    onClose={closeDialog}
    onSaved={refreshData}
/>

<AddStudentSubjectDialog
    open={activeDialog?.type === 'addSubject'}
    studentId={activeDialog?.type === 'addSubject' ? activeDialog.studentId : 0}
    onClose={closeDialog}
    onSaved={refreshData}
/>

<StudentsCsvImportDialog
    open={activeDialog?.type === 'importCsv'}
    onClose={closeDialog}
    onImported={refreshData}
/>

<Snackbar />

<style>
    main {
        padding: 0;
        display: block !important;
    }

    main > form {
        display: grid;
        grid-template-columns: minmax(220px, 1fr) minmax(160px, 220px);
        gap: 0.75rem;
        margin-bottom: 1rem;
    }

    .students-grid :global(.wx-scroll) {
        scrollbar-gutter: stable;
    }

    @media (max-width: 40rem) {
        main > form {
            grid-template-columns: 1fr;
        }
    }
</style>
