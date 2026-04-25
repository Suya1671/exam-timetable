import { Temporal } from '@js-temporal/polyfill'
import { check, integer, minLength, minValue, number, pipe, string, transform } from 'valibot'

export function getDatesBetween(
    start: Temporal.PlainDate,
    end: Temporal.PlainDate,
): Temporal.PlainDate[] {
    if (Temporal.PlainDate.compare(end, start) < 0) {
        throw new Error('End date must be after start date')
    }

    const days = start.until(end).days

    return Array.from({ length: days + 1 }, (_, i) =>
        start.add({ days: i }))
}

export const gradesParser = pipe(
    string(),
    minLength(1, 'Provide at least one grade between 1 and 12'),
    transform(raw =>
        raw
            .split(',')
            .map(segment => segment.trim())
            .filter(segment => segment.length > 0)
            .map(segment => Number(segment)),
    ),
    minLength(1, 'Provide at least one grade between 1 and 12'),
    check(
        grades => grades.every(grade => Number.isInteger(grade)),
        'All grades must be integers',
    ),
    check(
        grades => grades.every(grade => grade >= 1 && grade <= 12),
        'Grades must be between 1 and 12',
    ),
    transform(grades => [...new Set(grades)]),
)

export const durationHoursValidator = pipe(
    number('Duration must be a number'),
    minValue(0.5, 'Duration must be at least 0.5 hours'),
)

export const slotsRequiredValidator = pipe(
    number('Slots required must be a number'),
    integer('Slots required must be an integer'),
    minValue(1, 'Slots required must be at least 1'),
)

export const priorityValidator = pipe(
    number('Priority must be a number'),
    integer('Priority must be an integer'),
    minValue(0, 'Priority must be at least 0'),
)
