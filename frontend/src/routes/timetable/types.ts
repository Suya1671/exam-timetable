/** AI-generated (GPT-5.3-codex). */
export type TimetableGrade = {
	value: number;
	label: string;
};

/** AI-generated (GPT-5.3-codex). */
export type TimetableExamEntry = {
	sessionId: number;
	examId: number;
	grade: number;
	label: string;
	timeRange: string;
	locked: boolean;
	subjectFamily: string;
};

/** AI-generated (GPT-5.3-codex). */
export type TimetableSessionRow = {
	label: string;
	timeslotId: number;
	examsByGrade: TimetableExamEntry[][];
};

/** AI-generated (GPT-5.3-codex). */
export type TimetableDay = {
	dateLabel: string;
	weekKey: string;
	sessions: TimetableSessionRow[];
};
