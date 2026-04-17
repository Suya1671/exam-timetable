import { check, integer, minLength, minValue, number, pipe, string, transform } from 'valibot'

/** AI-generated (GPT-5.2-codex). */
export function getDatesBetween(start: Date, end: Date): Date[] {
    const startUTC = Date.UTC(start.getFullYear(), start.getMonth(), start.getDate())
    const endUTC = Date.UTC(end.getFullYear(), end.getMonth(), end.getDate())

    if (endUTC < startUTC) {
        throw new Error('End date must be after start date')
    }

    const dayMs = 24 * 60 * 60 * 1000
    const length = Math.floor((endUTC - startUTC) / dayMs) + 1

    return Array.from({ length }, (_, i) => new Date(startUTC + i * dayMs))
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
