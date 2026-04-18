import type { ICellProps } from '@svar-ui/svelte-grid'

export interface Subject {
    id: number
    name: string
}

export interface StudentRow {
    id: number
    name: string
    grade: number
    subjects: Subject[]
}

export type StudentsActiveDialog
    = | { type: 'addSubject', studentId: number }
        | { type: 'addStudent' }
        | { type: 'editStudent', studentId: number, name: string, grade: number }
        | { type: 'importCsv' }
        | null

export interface AddSubjectEvent {
    studentId: number
}

export interface RemoveSubjectEvent {
    studentId: number
    subjectId: number
    subjectName?: string
}

export interface EditStudentEvent {
    studentId: number
    studentName?: string
    studentGrade: number
}

export interface DeleteStudentEvent {
    studentId: number
    studentName?: string
}

export interface AddAllSubjectsEvent {
    studentId: number
}

export interface ClearSubjectsEvent {
    studentId: number
    studentName?: string
}

export interface AvailableSubjectRow {
    id: number
    name: string
}

export type SubjectChipsCellProps = ICellProps & {
    row: StudentRow
}

export type StudentActionsCellProps = ICellProps & {
    row: Pick<StudentRow, 'id' | 'name' | 'grade'>
}
