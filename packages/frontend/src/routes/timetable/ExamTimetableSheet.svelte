<script lang='ts'>
    import type { TimetableDay } from '@exam-timetable/tauri-api'

    import type { ComponentProps } from 'svelte'
    import { RestrictToVerticalAxis } from '@dnd-kit/abstract/modifiers'

    import { RestrictToElement } from '@dnd-kit/dom/modifiers'
    import {
        createDraggable,
        createDroppable,
        DragDropProvider,
        KeyboardSensor,
        PointerSensor,
    } from '@dnd-kit/svelte'

    type DnDProviderProps = ComponentProps<typeof DragDropProvider>

    const {
        schoolName = 'School of The Crab',
        title = '',
        grades = [],
        days = [],
        onMoveExam,
        onToggleLock,
        onEditLabel,
        onEditTimes,
    }: {
        schoolName?: string
        title?: string
        grades?: number[]
        days?: TimetableDay[]
        onMoveExam?: (sessionId: number, timeslotId: number, grade: number) => void
        onToggleLock?: (sessionId: number) => void
        onEditLabel?: (sessionId: number) => void
        onEditTimes?: (sessionId: number) => void
    } = $props()

    const sensors = [PointerSensor, KeyboardSensor]

    let containerRef: HTMLElement | null = null
    const modifiers = $derived([
        RestrictToVerticalAxis,
        RestrictToElement.configure({ element: containerRef }),
    ])

    const handleDragEnd: DnDProviderProps['onDragEnd'] = (event) => {
        const op = event.operation
        const source = op?.source
        const target = op?.target
        if (!target || event.canceled)
            return
        if (!source || !source.data)
            return

        if (target.data.grade !== source.data.grade)
            return

        onMoveExam?.(source.data.sessionId, target.data.timeslotId, target.data.grade)
    }

    /** AI-generated (GPT-5.3-codex). */
    function familyColor(subjectFamily: string): string {
        let hash = 0
        for (let i = 0; i < subjectFamily.length; i += 1) {
            hash = (hash * 31 + subjectFamily.charCodeAt(i)) >>> 0
        }
        const hue = hash % 360
        return `hsl(${hue} 35% 42%)`
    }

    function createExamDraggable(sessionId: number, gradeValue: number, locked: boolean) {
        return createDraggable({
            id: `exam-${sessionId}`,
            disabled: locked,
            data: { sessionId, grade: gradeValue },
        })
    }

    function createCellDroppable(timeslotId: number, grade: number) {
        return createDroppable({
            id: `${timeslotId}-${grade}`,
            data: { timeslotId, grade },
            accept: source => source.data.grade === grade,
        })
    }

    const dateFormatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'long' })
</script>

<DragDropProvider {modifiers} {sensors} onDragEnd={handleDragEnd}>
    <section class='sheet' aria-label='Exam timetable'>
        <header>
            <h2>{schoolName}</h2>
            <h1>{title}</h1>
        </header>

        <div class='table-wrap'>
            <table>
                <thead>
                    <tr>
                        <th class='date-col'>Date</th>
                        <th class='session-col' aria-label='Session'></th>
                        {#each grades as grade}
                            <th>Grade {grade}</th>
                        {/each}
                    </tr>
                </thead>
                <tbody bind:this={containerRef}>
                    {#each days as day, dayIndex (`${day.date}-${day.weekNumber}`)}
                        {@const isWeekStart
                            = dayIndex === 0 || day.weekNumber !== days[dayIndex - 1]!.weekNumber}
                        {#if isWeekStart}
                            <tr class='week-row'>
                                <th colspan={grades.length + 2}>Week {day.weekNumber}</th>
                            </tr>
                        {/if}
                        {#each day.sessions as session, sessionIndex (session.timeslotId)}
                            <tr>
                                {#if sessionIndex === 0}
                                    <th
                                        scope='rowgroup'
                                        rowspan={day.sessions.length}
                                        class='date-label'
                                    >
                                        {dateFormatter.format(new Date(day.date))}
                                    </th>
                                {/if}
                                <th scope='row' class='session-label'
                                >Session {session.sessionNumber}</th
                                >
                                {#each grades as grade}
                                    {@const examEntries = session.exams.filter(
                                        e => e.grade === grade,
                                    )}
                                    {@const droppable = createCellDroppable(
                                        session.timeslotId,
                                        grade,
                                    )}
                                    <td
                                        {@attach droppable.attach}
                                        class:drop-target={droppable.isDropTarget}
                                    >
                                        <ul class='cell-content'>
                                            {#each examEntries as entry (entry.sessionId)}
                                                {@const draggable = createExamDraggable(
                                                    entry.sessionId,
                                                    grade,
                                                    entry.locked,
                                                )}
                                                <li>
                                                    <article
                                                        class='exam-chip'
                                                        class:dragging={draggable.isDragging}
                                                        style={entry.subject
                                                            ? `--family-accent: ${familyColor(entry.subject)}`
                                                            : ''}
                                                        {@attach draggable.attach}
                                                    >
                                                        <button
                                                            type='button'
                                                            class='drag-handle no-print'
                                                            disabled={entry.locked}
                                                            aria-label='Drag exam'
                                                            {@attach draggable.attachHandle}
                                                        >
                                                            ::
                                                        </button>
                                                        <p>{entry.subject}</p>
                                                        <p>
                                                            {#if entry.examName}
                                                                {entry.examName}
                                                            {:else}
                                                                Paper {entry.paperNumber}
                                                            {/if}
                                                        </p>
                                                        <p class='time-range'>
                                                            {entry.startTime} - {entry.endTime}
                                                        </p>
                                                        <menu
                                                            class='exam-actions no-print'
                                                            aria-label='Exam actions'
                                                        >
                                                            <li>
                                                                <button
                                                                    type='button'
                                                                    onclick={() =>
                                                                        onToggleLock?.(
                                                                            entry.sessionId,
                                                                        )}
                                                                >
                                                                    {entry.locked
                                                                        ? 'Unlock'
                                                                        : 'Lock'}
                                                                </button>
                                                            </li>
                                                            <li>
                                                                <button
                                                                    type='button'
                                                                    onclick={() =>
                                                                        onEditLabel?.(
                                                                            entry.sessionId,
                                                                        )}
                                                                >
                                                                    Edit
                                                                </button>
                                                            </li>
                                                            <li>
                                                                <button
                                                                    type='button'
                                                                    onclick={() =>
                                                                        onEditTimes?.(
                                                                            entry.sessionId,
                                                                        )}
                                                                >
                                                                    Times
                                                                </button>
                                                            </li>
                                                        </menu>
                                                    </article>
                                                </li>
                                            {/each}
                                        </ul>
                                    </td>
                                {/each}
                            </tr>
                        {/each}
                    {/each}
                </tbody>
            </table>
        </div>
    </section>
</DragDropProvider>

<style>
    .sheet {
        border: 2px solid #1a1a1a;
        padding: 1rem;
        background: #f1f1f1;
    }

    header {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
        gap: 1rem;
        margin-bottom: 0.5rem;
    }

    h1,
    h2 {
        margin: 0;
        font-family: 'IBM Plex Sans', 'Segoe UI', sans-serif;
        color: #111;
    }

    h1 {
        font-size: clamp(1.2rem, 2.6vw, 2rem);
        font-weight: 600;
    }

    h2 {
        font-size: clamp(1.1rem, 2.2vw, 1.8rem);
        font-weight: 500;
        text-decoration: underline;
    }

    .table-wrap {
        overflow-x: auto;
    }

    table {
        width: 100%;
        min-width: 66rem;
        border-collapse: collapse;
        font-family: 'IBM Plex Sans', 'Segoe UI', sans-serif;
        font-size: 1rem;
        line-height: 1.25;
        background: #f7f7f7;
    }

    th,
    td {
        border: 2px solid #222;
        padding: 0.35rem 0.45rem;
        vertical-align: top;
        text-align: center;
    }

    thead th {
        font-weight: 700;
        text-decoration: underline;
    }

    .date-col,
    .date-label {
        width: 8rem;
        font-weight: 700;
        vertical-align: middle;
    }

    .session-col,
    .session-label {
        width: 7.5rem;
        font-weight: 700;
        vertical-align: middle;
    }

    .session-label,
    .date-label {
        background: #ececec;
    }

    .cell-content {
        display: grid;
        gap: 0.5rem;
        min-height: 1.75rem;
        list-style: none;
        padding: 0;
    }

    td.drop-target {
        background: #c7d2fe;
        outline: 3px solid #6366f1;
    }

    .exam-chip {
        padding: 0.45rem 0.5rem;
        text-align: left;
        white-space: pre-line;
        border: 1px solid #aeb4b8;
        border-left: 4px solid var(--family-accent, #4f6d7a);
        border-radius: 0.35rem;
        background: #f8fafb;
        box-shadow: 0 1px 0 rgba(0, 0, 0, 0.06);
    }

    .exam-chip.dragging {
        opacity: 0.5;
    }

    .exam-chip p {
        margin: 0;
    }

    .exam-chip p:first-of-type {
        font-weight: 600;
    }

    .time-range {
        font-size: 0.95rem;
    }

    .exam-actions {
        display: flex;
        gap: 0.35rem;
        margin-top: 0.35rem;
        list-style: none;
        padding: 0;
    }

    .exam-actions button {
        border: 1px solid #2f2f2f;
        background: #fff;
        padding: 0.1rem 0.35rem;
        border-radius: 0.25rem;
        cursor: pointer;
        font-size: 0.8rem;
    }

    .drag-handle {
        border: 1px solid #2f2f2f;
        background: #f1f3f4;
        border-radius: 0.2rem;
        padding: 0 0.25rem;
        margin-right: 0.4rem;
        font-family: ui-monospace, monospace;
        cursor: grab;
    }

    .exam-actions button {
        background: #f3f3f3;
    }

    .week-row th {
        border-top-width: 3px;
        border-bottom-width: 3px;
        background: #e7e7e7;
        font-weight: 700;
        text-align: left;
        padding-left: 0.6rem;
    }

    @media (max-width: 70rem) {
        table {
            font-size: 0.92rem;
            min-width: 58rem;
        }
    }

    @media print {
        .sheet {
            padding: 0;
            border: 1px solid #000;
            background: #fff;
        }

        .exam-chip {
            border: 0;
            border-radius: 0;
            box-shadow: none;
            background: transparent;
            padding: 0.05rem 0;
        }

        table {
            min-width: 0;
            font-size: 10pt;
        }

        .no-print {
            display: none;
        }

        .week-row {
            break-inside: avoid;
        }
    }
</style>
