import type { Temporal } from '@js-temporal/polyfill'

export function buildMonthGrid(date: Temporal.PlainDate) {
    const first = date.with({ day: 1 })
    const last = first.add({ months: 1 }).subtract({ days: 1 })

    const start = first.subtract({ days: (first.dayOfWeek + 6) % 7 })

    const end = last.add({
        days: 6 - ((last.dayOfWeek + 6) % 7),
    })

    const total = end.since(start).days + 1

    return Array.from({ length: total }, (_, i) => {
        const d = start.add({ days: i })

        return {
            date: d,
            inMonth: d.month === first.month,
        }
    })
}
