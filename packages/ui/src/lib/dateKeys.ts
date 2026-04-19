export const dateKeyUTC = (date: Date): string => date.toISOString().slice(0, 10)

export function normalizeDateKey(value: string): string {
    const match = /^(\d{4}-\d{2}-\d{2})/.exec(value)
    return match?.[1] ?? value
}

export function dateFromKeyUTC(value: string): Date {
    const [year, month, day] = value.split('-')
    return new Date(Date.UTC(Number(year), Number(month) - 1, Number(day)))
}
