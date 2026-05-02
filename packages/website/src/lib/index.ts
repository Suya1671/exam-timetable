import type { TimetableData, TimetableDay, TimetableExamEntry, TimetableSession } from '@exam-timetable/tauri-api'
import type { BaseSchema } from 'valibot'
import { Temporal } from '@js-temporal/polyfill'
import { array, boolean, integer, nullable, number, object, pipe, string, transform } from 'valibot'

function defineSchema<T>() {
    // eslint-disable-next-line ts/no-explicit-any
    return <S extends { [K in keyof T]: BaseSchema<T[K], any, any> }>(
        schema: S,
    ) => object(schema)
}

const examSchema = defineSchema<TimetableExamEntry>()({
    examId: number(),
    sessionId: number(),
    subject: string(),
    grade: pipe(
        number(),
        integer(),
    ),
    paperNumber: pipe(
        number(),
        integer(),
    ),
    examName: nullable(string()),
    startTime: pipe(
        string(),
        transform(v => Temporal.PlainTime.from(v)),
    ),
    endTime: pipe(
        string(),
        transform(v => Temporal.PlainTime.from(v)),
    ),
    locked: boolean(),
})

const timetableSessionSchema = defineSchema<TimetableSession>()({
    timeslotId: number(),
    sessionNumber: number(),
    exams: array(examSchema),
})

const timetableDaySchema = defineSchema<TimetableDay>()({
    date: pipe(
        string(),
        transform(value =>
            Temporal.PlainDate.from(
                Temporal.Instant.from(value).toZonedDateTimeISO('UTC')
                    .toPlainDate(),
            ),
        ),
    ),
    sessions: array(timetableSessionSchema),
})

export const examTimetableSchema = defineSchema<TimetableData>()({
    title: string(),
    schoolName: string(),
    grades: array(pipe(
        number(),
        integer(),
    )),
    days: array(timetableDaySchema),
})
