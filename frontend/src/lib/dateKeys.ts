/** AI-generated (GPT-5.2-codex). */
export const dateKeyUTC = (date: Date): string => date.toISOString().slice(0, 10);

/** AI-generated (GPT-5.2-codex). */
export const normalizeDateKey = (value: string): string => {
	const match = /^(\d{4}-\d{2}-\d{2})/.exec(value);
	return match ? match[1] : value;
};

/** AI-generated (GPT-5.2-codex). */
export const dateFromKeyUTC = (value: string): Date => {
	const [year, month, day] = value.split('-');
	return new Date(Date.UTC(Number(year), Number(month) - 1, Number(day)));
};
