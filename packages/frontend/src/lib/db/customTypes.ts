import { Temporal } from '@js-temporal/polyfill'
import { customType } from 'drizzle-orm/sqlite-core'

export const customDate = customType<{ data: Temporal.PlainDate, driverData: string }>({
    dataType: () => 'string',
    toDriver(value) {
        return value.toString()
    },
    fromDriver(value) {
        return Temporal.PlainDate.from(value)
    },
})

export const customTime = customType<{ data: Temporal.PlainTime, driverData: string }>({
    dataType: () => 'string',
    toDriver(value) {
        return value.toString()
    },
    fromDriver(value) {
        return Temporal.PlainTime.from(value)
    },
})

// TODO: refactor durations to use Temporal.Duration rather than ints
// Requires rust/db refactor to use Strings
