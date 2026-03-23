import { customType } from "drizzle-orm/sqlite-core";

export const customDate = customType<{ data: Date; driverData: string }>({
    dataType: () => "string",
    toDriver(value) {
        return value.toISOString();
    },
    fromDriver(value) {
        return new Date(value);
    },
});
