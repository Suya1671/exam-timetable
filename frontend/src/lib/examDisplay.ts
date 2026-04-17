interface ExamLike {
    id?: number
    paper: number
    grade?: number | null
    name?: string | null
    subject?: {
        name?: string | null
    } | null
}

/**
 * Builds a consistent display label for exam-like objects across the frontend.
 */
export function formatExamLabel(exam: ExamLike): string {
    const subjectName = exam.subject?.name?.trim()
    const examName = exam.name?.trim()
    const hasGrade = exam.grade !== null && exam.grade !== undefined

    if (subjectName && examName) {
        return `${subjectName} ${examName} (Paper ${exam.paper})`
    }

    if (subjectName && hasGrade) {
        return `${subjectName} Grade ${exam.grade} Paper ${exam.paper}`
    }

    if (subjectName) {
        return `${subjectName} Paper ${exam.paper}`
    }

    if (hasGrade) {
        return `Grade ${exam.grade} Paper ${exam.paper}`
    }

    if (exam.id !== undefined) {
        return `Exam ${exam.id} Paper ${exam.paper}`
    }

    return `Paper ${exam.paper}`
}
