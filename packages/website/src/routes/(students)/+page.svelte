<script lang='ts'>
    import type { PDFDocument } from '@cantoo/pdf-lib'
    import type { ICalEventData } from 'ical-generator'
    import type { InferOutput } from 'valibot'
    import { examTimetableSchema } from '$lib'
    import setTypst from '$lib/setInitOptions'

    import TypstPreview from '$lib/TypstPreview.svelte'
    import { Temporal } from '@js-temporal/polyfill'

    import RemoveIcon from '@ktibow/iconset-material-symbols/close-rounded'
    import PlaylistAddIcon from '@ktibow/iconset-material-symbols/playlist-add-rounded'
    import UploadIcon from '@ktibow/iconset-material-symbols/upload-rounded'
    import ical, { ICalCalendarMethod, ICalEventBusyStatus, ICalEventClass, ICalEventStatus } from 'ical-generator'
    import { Button, Card, Chip, Icon, SelectOutlined } from 'm3-svelte'
    import { FileUpload } from 'melt/builders'
    import { safeParse } from 'valibot'
    import template from './student_template.typ?raw'

    const typst = await setTypst()

    let pdfReset = $state(() => {})
    const examPdf = new FileUpload({
        accept: '.pdf',
        onAccept: () => pdfReset(),
    })

    const loadPDFFromFile = async (file: File) => {
        const bytes = await file.arrayBuffer()

        const { PDFDocument } = await import('@cantoo/pdf-lib')
        return await PDFDocument.load(bytes)
    }

    const readTimetableData = (pdf: PDFDocument) => {
        const attachments = pdf.getAttachments()
        const dataAttachment = attachments.find(attachment => attachment.name === 'data.json')

        if (!dataAttachment) {
            throw new Error('No data.json attachment found in PDF. Are you sure this is a exam timetable PDF?')
        }

        const dataBytes = dataAttachment.data

        const decoded = JSON.parse(new TextDecoder().decode(dataBytes))
        return safeParse(examTimetableSchema, decoded)
    }

    let grade = $state('')
    let selectedSubjects = $state<string[]>([])

    const createFilteredTimetableData = (timetable: InferOutput<typeof examTimetableSchema>) => {
        const days = timetable.days.map(day => ({
            ...day,
            sessions: day.sessions.map((session) => {
                const exams = session.exams.filter(exam => selectedSubjects.includes(exam.subject) && exam.grade === Number(grade))
                if (exams.length > 1) {
                    throw new Error(`Multiple exams found on ${day.date.toLocaleString()} session ${session.sessionNumber}. You cannot write multiple exams at the same time. Exams: ${exams.map(exam => `${exam.subject} P${exam.paperNumber}`).join(', ')}`)
                }
                const exam = exams[0]

                return {
                    number: session.sessionNumber,
                    exam: exam ?? null,
                }
            }),
        }))

        const startDay = days.findIndex(day => day.sessions.some(session => session.exam !== null))
        const endDay = days.findLastIndex(day => day.sessions.some(session => session.exam !== null))

        const filteredDays = days.slice(startDay, endDay + 1)

        return {
            title: timetable.title,
            schoolName: timetable.schoolName,
            grade: Number(grade),
            days: filteredDays,
        }
    }

    const exportPdf = async (data: ReturnType<typeof createFilteredTimetableData>) => {
        const pdf = await typst.pdf({ mainContent: template, inputs: { data: JSON.stringify(data) } })
        if (!pdf)
            throw new Error('Failed to generate PDF')

        const blob = new Blob([pdf as Uint8Array<ArrayBuffer>], { type: 'application/pdf' })
        const url = URL.createObjectURL(blob)
        const link = document.createElement('a')
        link.href = url
        link.download = 'timetable.pdf'
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        URL.revokeObjectURL(url)
    }

    const hasMultipleExams = (data: ReturnType<typeof createFilteredTimetableData>, subject: string) => {
        return data.days.flatMap(day => day.sessions).filter(session => session.exam?.subject === subject).length > 1
    }

    function generateICS(data: ReturnType<typeof createFilteredTimetableData>) {
        // TODO: rather embed the timezone into the generated main calendar
        const userTZ = Temporal.Now.timeZoneId()

        const events = data.days.flatMap((day) => {
            return day.sessions.filter(session => session.exam).map((session) => {
                const exam = session.exam!

                const start = day.date.toZonedDateTime({ timeZone: userTZ, plainTime: exam.startTime })
                const end = day.date.toZonedDateTime({ timeZone: userTZ, plainTime: exam.endTime })

                const examName
                    = exam.examName
                        ?? (hasMultipleExams(data, exam.subject)
                            ? `Paper ${exam.paperNumber}`
                            : '')

                // TODO: automatically create alarms
                return {
                    id: `${exam.examId}-${exam.sessionId}@exam-timetable`,
                    start,
                    end,
                    location: data.schoolName,
                    class: ICalEventClass.PUBLIC,
                    status: ICalEventStatus.CONFIRMED,
                    busystatus: ICalEventBusyStatus.BUSY,
                    summary: `${exam.subject} ${examName}`,
                    description: exam.examName ?? '',
                    categories: [{ name: 'Exam' }],
                } satisfies ICalEventData
            })
        })

        const cal = ical({
            method: ICalCalendarMethod.PUBLISH,
            prodId: '-//Exam Timetable//EN',
            name: `${data.title}`,
            description: `Exams for ${data.title} at ${data.schoolName}`,
            url: 'https://timetable.wobbl.in',
            events,
        })

        return cal.toString()
    }

    const exportToCalendar = (data: ReturnType<typeof createFilteredTimetableData>) => {
        const ics = generateICS(data)
        const blob = new Blob([ics], { type: 'text/calendar;charset=utf-8' })
        const url = URL.createObjectURL(blob)

        window.location.href = url

        // Cleanup. No clue if theres a better way to do this
        setTimeout(() => URL.revokeObjectURL(url), 5000)
    }
</script>

<svelte:head>
    <title>Personalised Timetable Generator | Exam Timetable</title>
    <meta name='description' content='Generate your personalised exam timetable with ease from a timetable created by exam-timetable.' />
</svelte:head>

<header>
    <h1>Exam Timetable: Student Timetable Generator</h1>
</header>

<main>
    <input {...examPdf.input} />
    <Card {...examPdf.dropzone} variant={examPdf.isDragging ? 'elevated' : 'filled'}>
        <Icon icon={UploadIcon} size={64}></Icon>

        <p>
            {#if examPdf.selected}
                {examPdf.selected.name}
            {:else if examPdf.isDragging}
                Drop your PDF here
            {:else}
                Click to upload your exam timetable PDF <span class='subtle'>or drag and drop</span>
            {/if}
        </p>
    </Card>

    {#if examPdf.selected}
        <svelte:boundary onerror={(_, reset) => (pdfReset = reset)}>
            {@const pdf = await loadPDFFromFile(examPdf.selected)}
            {@const timetable = readTimetableData(pdf)}

            {#if timetable.success}
                {@const data = timetable.output}
                {@const gradeOptions = data.grades.map(grade => ({ value: String(grade), text: `Grade ${grade}` }))}
                {@const examsForGrade = data.days.flatMap(day => day.sessions).flatMap(day => day.exams).filter(exam => exam.grade === Number(grade)).toSorted((a, b) => a.subject.localeCompare(b.subject))}
                {@const subjectsForGrade = Array.from(new Set(examsForGrade.map(exam => exam.subject)))}

                <h2>Creating timetable for {data.schoolName}: {data.title}</h2>

                <SelectOutlined
                    label='Grade'
                    options={gradeOptions}
                    bind:value={grade}
                    width='100%'
                    onchange={() => selectedSubjects = []}
                />

                <fieldset>
                    <legend>Subjects</legend>

                    <div class='subject-actions'>
                        <Chip
                            variant='assist'
                            disabled={subjectsForGrade.length === 0 || selectedSubjects.length === subjectsForGrade.length}
                            onclick={() => selectedSubjects = subjectsForGrade}
                            icon={PlaylistAddIcon}
                        >
                            Select all
                        </Chip>
                        <Chip
                            variant='assist'
                            disabled={selectedSubjects.length === 0}
                            onclick={() => selectedSubjects = []}
                            icon={RemoveIcon}
                        >
                            Clear
                        </Chip>
                    </div>

                    <ul class='subject-list'>
                        {#each subjectsForGrade as subject}
                            <li>
                                <Chip
                                    variant='input'
                                    selected={selectedSubjects.includes(subject)}
                                    onclick={() => selectedSubjects = selectedSubjects.includes(subject) ? selectedSubjects.filter((s: string) => s !== subject) : [...selectedSubjects, subject]}
                                >
                                    {subject}
                                </Chip>
                            </li>
                        {/each}
                    </ul>
                </fieldset>

                {#key grade + selectedSubjects.join(',')}
                    <svelte:boundary>
                        {@const filteredData = createFilteredTimetableData(data)}

                        {#if filteredData.days.length > 0}
                            {@const vectorData = await typst.vector({ mainContent: template, inputs: { data: JSON.stringify(filteredData) } })}

                            <div class='generated-timetable-header'>
                                <hgroup>
                                    <h2>Generated Timetable Preview</h2>
                                    <p>note: not fully accurate to final output</p>
                                </hgroup>

                                <div>
                                    <Button onclick={() => exportToCalendar(filteredData)}>
                                        Export to Calendar
                                    </Button>
                                    <Button onclick={() => exportPdf(filteredData)}>
                                        Export PDF
                                    </Button>
                                </div>
                            </div>

                            {#if vectorData}
                                <TypstPreview {vectorData} />
                            {:else}
                                <h2>No preview available</h2>
                            {/if}
                        {/if}

                        {#snippet pending()}
                            <h2>Generating preview...</h2>
                        {/snippet}

                        {#snippet failed(error)}
                            <p>Error generating preview: {error.message ?? JSON.stringify(error)}</p>
                        {/snippet}
                    </svelte:boundary>
                {/key}
            {:else}
                <p>Error validating timetable data! This means that this timetable is either invalid, corrupted, or outdated. Issues found:</p>

                <ul>
                    {#each timetable.issues as issue}
                        <li>
                            <h3>{issue.message}</h3>
                            <p>Path: {issue.path}</p>
                        </li>
                    {/each}
                </ul>
            {/if}

            {#snippet pending()}
                <h2>Loading...</h2>
            {/snippet}

            {#snippet failed(error)}
                <p>Error loading timetable: {error.message ?? JSON.stringify(error)}</p>
            {/snippet}
        </svelte:boundary>
    {/if}
</main>

<style>
    .subtle {
        opacity: 0.8;
    }

    .subject-actions,
    .subject-list {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
        padding: 0;

        li {
            list-style: none;
        }
    }

    main {
        display: flex;
        flex-direction: column;
        gap: 1rem;

        :global([data-melt-fileupload-dropzone]) {
            display: flex;
            flex-direction: column;
            width: 100%;
            outline: 2px dotted var(--m3c-outline);
            align-items: center;
            justify-content: center;
            cursor: pointer;
            gap: 0.5rem;
        }
    }

    fieldset {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;

        border: 1px solid var(--m3c-outline-variant);
        border-radius: var(--m3-shape-medium);
        background-color: var(--m3c-surface);
        padding: 1rem 1.5rem 1.5rem;
        margin: 0;

        & legend {
            padding-inline: 0.25rem;
            @apply --m3-label-large;
        }
    }

    .generated-timetable-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        margin-top: 1rem;

        h2 {
            @apply --m3-headline-medium;
        }

        p {
            @apply --m3-label-small;
        }

        div {
            display: flex;
            gap: 0.5rem;
        }
    }
</style>
