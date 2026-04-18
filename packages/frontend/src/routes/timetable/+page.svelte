<script lang='ts'>
    import type {
        InitSolverError,
        LockedTimetableSlot,
        NewTimetableUpdate,
        SolveSessionControlError,
        SolveSessionStart,
        TimeslotId,
        TimetableDay,
        TimetableSession,
    } from '@exam-timetable/tauri-api'
    import { dateKeyUTC } from '$lib/dateKeys'
    import { db } from '$lib/db'
    import { sessionTimeConfig, timetables, timetableSlots } from '$lib/db/schema'
    import { createTauRPCProxy } from '@exam-timetable/tauri-api'
    import { Temporal } from '@js-temporal/polyfill'
    import { eq } from 'drizzle-orm'
    import { Button } from 'm3-svelte'
    import { SvelteMap, SvelteSet } from 'svelte/reactivity'
    import ExamTimetableSheet from './ExamTimetableSheet.svelte'
    import ExportTimetableDialog from './ExportTimetableDialog.svelte'

    type SessionView = {
        sessionId: number
        examId: number
        grade: number
        subject: string
        durationHours: number
        paper: number
        examName: string | null
        subjectFamily: string
    }

    type SessionTimeOverride = {
        readingStartTime: Temporal.PlainTime | null
        examStartTime: Temporal.PlainTime | null
        examEndTime: Temporal.PlainTime | null
    }

    const backend = createTauRPCProxy()

    const initialSessions = await db.query.session.findMany({
        with: {
            exam: {
                with: {
                    subject: true,
                },
            },
        },
        orderBy: {
            id: 'asc',
        },
    })

    const initialTimeslots = await db.query.timeslot.findMany({
        orderBy: {
            date: 'asc',
            slot: 'asc',
        },
    })

    const initialGrades = await db.query.subjectGrade.findMany({
        columns: {
            grade: true,
        },
    })

    type SessionTimeRow = typeof sessionTimeConfig.$inferSelect
    type SessionTimesMap = Record<SessionTimeRow['slot'], SessionTimeRow>

    const sessionTimes = await db.query.sessionTimeConfig.findMany({
        orderBy: {
            slot: 'asc',
        },
    })

    const sessionTimeBySlot: SessionTimesMap = Object.fromEntries(
        sessionTimes.map(row => [row.slot, row]),
    )

    const initialSavedTimetables = await db.query.timetables.findMany({
        orderBy: {
            id: 'desc',
        },
    })

    const grades = $derived.by(() => {
        const uniqueGrades = new SvelteSet<number>()
        for (const row of initialGrades) {
            uniqueGrades.add(row.grade)
        }
        return [...uniqueGrades].sort((a, b) => a - b)
    })

    const sessions = $derived.by(() => {
        return initialSessions.map(row => ({
            sessionId: row.id,
            examName: row.exam.name,
            examId: row.examId,
            grade: row.exam.grade,
            subject: row.exam.subject.name,
            durationHours: row.exam.durationHours,
            paper: row.exam.paper,
            subjectFamily: row.exam.subject.name,
        })) as SessionView[]
    })

    let variants = $state<Array<SvelteMap<number, number>>>([])
    let variantIndex = $state(-1)
    let solving = $state(false)
    const manualAssignments = new SvelteMap<number, number>()
    const lockedSessions = new SvelteSet<number>()
    const timeOverrides = new SvelteMap<number, SessionTimeOverride>()
    const labelOverrides = new SvelteMap<number, string>()
    let savedTimetables = $state(initialSavedTimetables)
    let selectedTimetableId = $state<number | null>(null)

    const selectedVariant = $derived.by(
        () => variants[variantIndex] ?? new SvelteMap<number, number>(),
    )

    const assignments = $derived.by(() => {
        const merged = new SvelteMap<number, number>(selectedVariant)
        for (const [sessionId, timeslotId] of manualAssignments) {
            merged.set(sessionId, timeslotId)
        }
        return merged
    })

    /** AI-generated (GPT-5.3-codex). */
    function timeOverrideFor(sessionId: number) {
        return timeOverrides.get(sessionId) ?? null
    }

    /**
     * Gets the week number from 1-52 (ISO 8601).
     */
    const getWeekNumber = (date: Date) => {
        const week = Temporal.PlainDate.from(date.toISOString().split('T')[0]!).weekOfYear
        if (week === undefined)
            throw new Error(`weekOfYear is not set for date ${date.toISOString()}`)
        return week
    }

    const firstDay = $derived(
        new Date(Math.min(...initialTimeslots.map(timeslot => timeslot.date.getTime()))),
    )
    const firstWeek = $derived(getWeekNumber(firstDay))

    /** AI-generated (GPT-5.3-codex). Modified but not fully checked over */
    const days = $derived.by(() => {
        const byDate = new SvelteMap<string, TimetableDay>()

        for (const timeslot of initialTimeslots) {
            const key = dateKeyUTC(timeslot.date)
            const weekNumber = getWeekNumber(timeslot.date) - firstWeek + 1
            const sessionTime = sessionTimeBySlot[timeslot.slot]
            if (!sessionTime)
                throw new Error(`sessionTime for slot ${timeslot.slot} is not set`)
            const defaultReadingStart = sessionTime?.readingStartTime
            const defaultExamStart
                = sessionTime?.examStartTime ?? defaultReadingStart.add({ minutes: 15 })
            const row: TimetableSession = {
                sessionNumber: timeslot.slot + 1,
                timeslotId: timeslot.id,
                exams: [],
            }

            for (const session of sessions) {
                if (assignments.get(session.sessionId) !== timeslot.id)
                    continue
                const override = timeOverrideFor(session.sessionId)
                const readingStart = override?.readingStartTime ?? defaultReadingStart
                const examStart = override?.examStartTime ?? defaultExamStart
                const examEnd
                    = override?.examEndTime
                        ?? examStart.add({ minutes: Math.round(session.durationHours * 60) })

                row.exams.push({
                    sessionId: session.sessionId,
                    examId: session.examId,
                    grade: session.grade,
                    examName: session.examName,
                    subject: labelOverrides.get(session.sessionId) ?? session.subject,
                    startTime: readingStart.toLocaleString('en-ZA', {
                        hour: '2-digit',
                        minute: '2-digit',
                    }),
                    endTime: examEnd.toLocaleString('en-ZA', {
                        hour: '2-digit',
                        minute: '2-digit',
                    }),
                    paperNumber: session.paper,
                    locked: lockedSessions.has(session.sessionId),
                })
            }

            row.exams.sort((a, b) => {
                const gradeDiff = a.grade - b.grade
                if (gradeDiff !== 0)
                    return gradeDiff
                const subjectDiff = a.subject.localeCompare(b.subject)
                if (subjectDiff !== 0)
                    return subjectDiff
                return a.paperNumber - b.paperNumber
            })

            const existing = byDate.get(key)
            if (!existing) {
                byDate.set(key, { date: timeslot.date.toISOString(), weekNumber, sessions: [row] })
            }
            else {
                existing.sessions.push(row)
            }
        }

        return [...byDate.values()]
    })

    /** AI-generated (GPT-5.3-codex). */
    function parseSolution(solution: Record<number, TimeslotId>) {
        const map = new SvelteMap<number, number>()
        for (const [sessionId, timeslotId] of Object.entries(solution)) {
            map.set(Number(sessionId), timeslotId)
        }
        return map
    }

    let solveSessionId = $state<number | null>(null)
    let solveDone = $state(false)
    let streamPaused = $state(false)
    let showExportDialog = $state(false)

    /** AI-generated (GPT-5.3-codex). */
    function handleSolveUpdate(update: NewTimetableUpdate) {
        if (update === 'Done') {
            solveDone = true
            solveSessionId = null
            streamPaused = false
            return
        }

        if ('Timetable' in update) {
            // typesafety: we know that the key and the value always exist as per the rust-side arguments
            variants.push(parseSolution(update.Timetable as Record<number, TimeslotId>))
            if (variants.length === 1) {
                variantIndex = 0
            }
        }
    }

    /** AI-generated (GPT-5.3-codex). */
    function describeInitSolverError(error: InitSolverError): string {
        if ('LockPoison' in error)
            return error.LockPoison
        if ('Solver' in error)
            return error.Solver
        if ('Recv' in error)
            return error.Recv
        return 'Unknown solver startup error'
    }

    /** AI-generated (GPT-5.3-codex). */
    function describeSolveSessionControlError(error: SolveSessionControlError): string {
        if ('LockPoison' in error)
            return error.LockPoison
        if ('Send' in error)
            return error.Send
        if ('InvalidSessionId' in error)
            return `Invalid session id ${error.InvalidSessionId}`
        return 'Unknown solver control error'
    }

    /** AI-generated (GPT-5.3-codex). */
    function buildLockedSlots(): LockedTimetableSlot[] {
        const lockedSlots: LockedTimetableSlot[] = []

        for (const sessionId of lockedSessions) {
            const timeslotId = assignments.get(sessionId)
            if (timeslotId === undefined)
                continue
            lockedSlots.push({
                session_id: sessionId,
                timeslot_id: timeslotId,
            })
        }

        return lockedSlots
    }

    /** AI-generated (GPT-5.3-codex). */
    function nextTimetableName(): string {
        return `Timetable ${savedTimetables.length + 1}`
    }

    /** AI-generated (GPT-5.3-codex). */
    async function refreshSavedTimetables() {
        savedTimetables = await db.query.timetables.findMany({
            orderBy: {
                id: 'desc',
            },
        })
    }

    /** AI-generated (GPT-5.3-codex). */
    async function saveCurrentTimetable() {
        const now = new Date().toISOString()
        if (selectedTimetableId === null) {
            const [created] = await db
                .insert(timetables)
                .values({
                    name: nextTimetableName(),
                    createdAt: now,
                    updatedAt: now,
                })
                .returning({ id: timetables.id })
            selectedTimetableId = created?.id ?? null
        }

        if (selectedTimetableId === null)
            return

        const rows: Array<typeof timetableSlots.$inferInsert> = []
        for (const [sessionId, timeslotId] of assignments) {
            const override = timeOverrideFor(sessionId)
            rows.push({
                timetableId: selectedTimetableId,
                sessionId,
                timeslotId,
                locked: lockedSessions.has(sessionId),
                readingStartTime: override?.readingStartTime,
                examStartTime: override?.examStartTime,
                examEndTime: override?.examEndTime,
            })
        }

        await db.transaction(async (tx) => {
            await tx
                .delete(timetableSlots)
                .where(eq(timetableSlots.timetableId, selectedTimetableId!))
            if (rows.length > 0) {
                await tx.insert(timetableSlots).values(rows)
            }
            await tx
                .update(timetables)
                .set({ updatedAt: now })
                .where(eq(timetables.id, selectedTimetableId!))
        })

        await refreshSavedTimetables()
    }

    /** AI-generated (GPT-5.3-codex). */
    async function loadSavedTimetable(id: number) {
        const rows = await db.query.timetableSlots.findMany({
            where: {
                timetableId: id,
            },
        })

        manualAssignments.clear()
        lockedSessions.clear()
        timeOverrides.clear()
        labelOverrides.clear()

        for (const row of rows) {
            manualAssignments.set(row.sessionId, row.timeslotId)
            if (row.locked) {
                lockedSessions.add(row.sessionId)
            }
            timeOverrides.set(row.sessionId, row)
        }

        selectedTimetableId = id
    }

    /** AI-generated (GPT-5.3-codex). */
    async function selectSavedTimetable(event: Event) {
        const target = event.currentTarget as HTMLSelectElement
        const nextId = Number(target.value)
        if (!Number.isFinite(nextId) || nextId <= 0) {
            selectedTimetableId = null
            resetEdits()
            return
        }
        await loadSavedTimetable(nextId)
    }

    /** AI-generated (GPT-5.3-codex). */
    async function solve() {
        solving = true
        variants = []
        variantIndex = -1
        solveDone = false
        streamPaused = false

        if (solveSessionId !== null) {
            try {
                await backend.stop_solve_session(solveSessionId)
            }
            catch (error) {
                console.error('Failed to stop previous solve session', error)
            }
            solveSessionId = null
        }

        try {
            const started: SolveSessionStart = await backend.start_solve_session(
                handleSolveUpdate,
                buildLockedSlots(),
            )
            solveSessionId = started.session_id
        }
        catch (e) {
            const error = e as InitSolverError
            console.error('Error during solving:', error)
            alert(`An error occurred while starting solve: ${describeInitSolverError(error)}`)
            solveSessionId = null
            solveDone = true
        }
        finally {
            solving = false
        }
    }

    /** AI-generated (minimax-m2.5). */
    async function solveOnce() {
        solving = true
        variants = []
        variantIndex = -1
        solveDone = false
        streamPaused = false

        if (solveSessionId !== null) {
            try {
                await backend.stop_solve_session(solveSessionId)
            }
            catch (error) {
                console.error('Failed to stop previous solve session', error)
            }
            solveSessionId = null
        }

        try {
            const result = await backend.solve_single(buildLockedSlots())
            // typesafety: we know that the key and the value always exist as per the rust-side arguments
            variants.push(parseSolution(result as Record<number, TimeslotId>))
            if (variants.length === 1) {
                variantIndex = 0
            }
            solveDone = true
        }
        catch (e) {
            const error = e as InitSolverError
            console.error('Error during solve once:', error)
            alert(`An error occurred while solving: ${describeInitSolverError(error)}`)
            solveDone = true
        }
        finally {
            solving = false
        }
    }

    /** AI-generated (GPT-5.3-codex). */
    function moveExam(sessionId: number, timeslotId: number, grade: number) {
        const session = sessions.find(session => session.sessionId === sessionId)
        if (!session || session.grade !== grade || lockedSessions.has(sessionId))
            return
        manualAssignments.set(sessionId, timeslotId)
    }

    /** AI-generated (GPT-5.3-codex). */
    function toggleLock(sessionId: number) {
        if (lockedSessions.has(sessionId)) {
            // eslint-disable-next-line drizzle/enforce-delete-with-where
            lockedSessions.delete(sessionId)
            return
        }
        lockedSessions.add(sessionId)
    }

    /** AI-generated (GPT-5.3-codex). */
    function editSubject(sessionId: number) {
        const session = sessions.find(session => session.sessionId === sessionId)
        if (!session)
            return
        const nextSubject = prompt(
            'Edit exam subject',
            labelOverrides.get(sessionId) ?? session.subject,
        )
        if (nextSubject === null)
            return
        const cleaned = nextSubject.trim()
        if (!cleaned || cleaned === session.subject) {
            // eslint-disable-next-line drizzle/enforce-delete-with-where
            labelOverrides.delete(sessionId)
        }
        else {
            labelOverrides.set(sessionId, cleaned)
        }
    }

    /** AI-generated (GPT-5.3-codex). */
    // TODO: make this a Dialog
    function editSessionTimeOverride(sessionId: number) {
        const current = timeOverrideFor(sessionId)
        const reading = prompt(
            'Reading start time (HH:MM)',
            current?.readingStartTime?.toString({ smallestUnit: 'minute' }),
        )
        if (reading === null)
            return
        const examStart = prompt(
            'Exam start time (HH:MM)',
            current?.examStartTime?.toString({ smallestUnit: 'minute' }) ?? '',
        )
        if (examStart === null)
            return
        const examEnd = prompt(
            'Exam end time (HH:MM)',
            current?.examEndTime?.toString({ smallestUnit: 'minute' }) ?? '',
        )
        if (examEnd === null)
            return

        timeOverrides.set(sessionId, {
            readingStartTime: Temporal.PlainTime.from(reading),
            examStartTime: Temporal.PlainTime.from(examStart),
            examEndTime: Temporal.PlainTime.from(examEnd),
        })
    }

    /** AI-generated (GPT-5.3-codex). */
    function prevVariant() {
        if (variantIndex > 0)
            variantIndex -= 1
    }

    /** AI-generated (GPT-5.3-codex). */
    function nextVariant() {
        if (variantIndex < variants.length - 1) {
            variantIndex += 1
        }
    }

    /** AI-generated (GPT-5.3-codex). */
    async function stopSolveSession() {
        if (solveSessionId === null)
            return
        try {
            await backend.stop_solve_session(solveSessionId)
        }
        catch (error) {
            const typed = error as SolveSessionControlError
            alert(`Failed to stop solve session: ${describeSolveSessionControlError(typed)}`)
        }
        solveSessionId = null
        solveDone = true
        streamPaused = false
    }

    /** AI-generated (GPT-5.3-codex). */
    function resetEdits() {
        manualAssignments.clear()
        lockedSessions.clear()
        timeOverrides.clear()
        labelOverrides.clear()
    }

    /** AI-generated (GPT-5.3-codex). */
    async function toggleStream() {
        if (solveSessionId === null || solveDone)
            return
        try {
            await backend.pause_solve_session(solveSessionId)
            streamPaused = !streamPaused
        }
        catch (error) {
            const typed = error as SolveSessionControlError
            alert(`Failed to change stream state: ${describeSolveSessionControlError(typed)}`)
        }
    }
</script>

<header class='no-print'>
    <h1>Manage Timetables</h1>
    <menu aria-label='Timetable actions'>
        <li>
            <label class='timetable-select'>
                <span>Saved</span>
                <select onchange={selectSavedTimetable} value={selectedTimetableId ?? ''}>
                    <option value="">Current draft</option>
                    {#each savedTimetables as saved (saved.id)}
                        <option value={saved.id}>{saved.name}</option>
                    {/each}
                </select>
            </label>
        </li>
        <li>
            <Button onclick={saveCurrentTimetable} disabled={assignments.size === 0}>Save</Button>
        </li>
        <li class='divider'></li>
        <li>
            <Button onclick={solveOnce} disabled={solving}
            >{solving ? 'Solving...' : 'Quick Solve'}</Button
            >
        </li>
        <li><Button onclick={solve} disabled={solving}>Stream</Button></li>
        {#if solveSessionId !== null && !solveDone}
            <li><Button onclick={toggleStream}>{streamPaused ? 'Resume' : 'Pause'}</Button></li>
            <li><Button onclick={stopSolveSession}>Stop</Button></li>
        {/if}
        <li class='divider'></li>
        <li><Button onclick={() => (showExportDialog = true)}>Export PDF</Button></li>
        <li><Button onclick={resetEdits}>Reset</Button></li>
    </menu>
</header>

<main>
    <section class='no-print' aria-label='Solution navigation and status'>
        <menu aria-label='Solution pager'>
            <li><Button onclick={prevVariant} disabled={variantIndex <= 0}>Previous</Button></li>
            <li>
                <p>
                    {#if variants.length === 0}
                        No streamed solutions yet
                    {:else}
                        Solution {variantIndex + 1} of {variants.length}
                    {/if}
                </p>
            </li>
            <li>
                <Button onclick={nextVariant} disabled={variantIndex >= variants.length - 1}
                >Next</Button
                >
            </li>
        </menu>
        <ul aria-label='Solver status'>
            <li>Locked exams: {lockedSessions.size}</li>
            <li>
                {#if solveDone}
                    Solver: done
                {:else if solveSessionId !== null}
                    Solver: active
                {:else}
                    Solver: idle
                {/if}
            </li>
            <li class='hint'>Lock and edit controls are hidden on print.</li>
        </ul>
    </section>

    <ExamTimetableSheet
        title={selectedTimetableId === null
            ? 'Draft timetable'
            : `Saved timetable ${selectedTimetableId}`}
        {grades}
        {days}
        onMoveExam={moveExam}
        onToggleLock={toggleLock}
        onEditLabel={editSubject}
        onEditTimes={editSessionTimeOverride}
    />

    <ExportTimetableDialog
        bind:open={showExportDialog}
        onClose={() => (showExportDialog = false)}
        title={selectedTimetableId === null
            ? 'Draft timetable'
            : `Saved timetable ${selectedTimetableId}`}
        {grades}
        {days}
    />
</main>

<style>
    section[aria-label='Solution navigation and status'],
    section[aria-label='Solution navigation and status'] > menu,
    section[aria-label='Solution navigation and status'] > ul,
    header > menu {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
    }

    menu {
        list-style: none;
        padding: 0;
    }

    li.divider {
        width: 2px;
        height: 1.5rem;
        background-color: oklch(70% 0 0);
        margin: 0 0.25rem;
    }

    .timetable-select {
        display: flex;
        align-items: center;
        gap: 0.35rem;
    }

    .timetable-select > span {
        font-size: 0.9rem;
    }

    .timetable-select select {
        padding: 0.25rem 0.35rem;
    }

    section[aria-label='Solution navigation and status'] > menu p,
    section[aria-label='Solution navigation and status'] > ul {
        margin: 0;
    }

    section[aria-label='Solution navigation and status'] > ul {
        list-style: none;
        padding: 0;
    }

    section[aria-label='Solution navigation and status'] {
        justify-content: space-between;
    }

    @media print {
        @page {
            size: landscape;
            margin: 8mm;
        }

        :global(body *) {
            visibility: hidden;
        }

        :global(.sheet),
        :global(.sheet *) {
            visibility: visible;
        }

        :global(.sheet) {
            position: absolute;
            left: 0;
            top: 0;
            width: 100%;
        }

        .no-print {
            display: none;
        }

        main {
            padding: 0;
        }

        :global(body) {
            background: #fff;
        }
    }
</style>
