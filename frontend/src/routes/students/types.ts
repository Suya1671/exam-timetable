import type { ICellProps } from '@svar-ui/svelte-grid';

export type Subject = {
	id: number;
	name: string;
};

export type StudentRow = {
	id: number;
	name: string;
	grade: number;
	subjects: Subject[];
};

export type StudentsActiveDialog =
	| { type: 'addSubject'; studentId: number }
	| { type: 'addStudent' }
	| { type: 'editStudent'; studentId: number; name: string; grade: number }
	| { type: 'importCsv' }
	| null;

export type AddSubjectEvent = {
	studentId: number;
};

export type RemoveSubjectEvent = {
	studentId: number;
	subjectId: number;
	subjectName?: string;
};

export type EditStudentEvent = {
	studentId: number;
	studentName?: string;
	studentGrade: number;
};

export type DeleteStudentEvent = {
	studentId: number;
	studentName?: string;
};

export type AddAllSubjectsEvent = {
	studentId: number;
};

export type ClearSubjectsEvent = {
	studentId: number;
	studentName?: string;
};

export type AvailableSubjectRow = {
	id: number;
	name: string;
};

export type SubjectChipsCellProps = ICellProps & {
	row: StudentRow;
};

export type StudentActionsCellProps = ICellProps & {
	row: Pick<StudentRow, 'id' | 'name' | 'grade'>;
};
