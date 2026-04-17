import { dateFromKeyUTC, dateKeyUTC, normalizeDateKey } from '$lib/dateKeys'
import { Temporal } from '@js-temporal/polyfill'
import { customType } from 'drizzle-orm/sqlite-core'

export const customDate = customType<{ data: Date, driverData: string }>({
    dataType: () => 'string',
    toDriver(value) {
        return dateKeyUTC(value)
    },
    fromDriver(value) {
        return dateFromKeyUTC(normalizeDateKey(value))
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
