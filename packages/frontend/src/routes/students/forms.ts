import { integer, maxValue, minValue, nonEmpty, number, pipe, string } from 'valibot'

export const MIN_GRADE = 1
export const MAX_GRADE = 12
export const DEFAULT_GRADE = 8

export const gradeValidator = pipe(
    number('Grade must be a number.'),
    integer('Grade must be an integer.'),
    minValue(MIN_GRADE, `Grade must be at least ${MIN_GRADE}.`),
    maxValue(MAX_GRADE, `Grade must be at most ${MAX_GRADE}.`),
)

export const studentNameValidator = pipe(string(), nonEmpty('Student name is required.'))
