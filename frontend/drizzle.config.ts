import { defineConfig } from "drizzle-kit";

export default defineConfig({
    dialect: "sqlite",
    out: "./src/lib/db",
    dbCredentials: {
        url: process.env.DRIZZLE_DB_URL ?? "../db.sqlite3",
    },
    introspect: {
        casing: "camel",
    },
    verbose: true,
    breakpoints: true,
});
