import {
    createTauRPCProxy,
    type SqlParam,
    type SqlQueryResult,
    type SqlValue,
} from "$lib/backend";
import { drizzle } from "drizzle-orm/sqlite-proxy";

import { relations } from "./relations";

const backend = createTauRPCProxy() as unknown as {
    sql: (
        sql: string,
        params: SqlParam[],
        method: string,
    ) => Promise<SqlQueryResult>;
};

function fromSqlValue(value: SqlValue) {
    if (value === "Null") {
        return null;
    }

    if ("Bool" in value) {
        return value.Bool;
    }

    if ("Int" in value) {
        return value.Int;
    }

    if ("Float" in value) {
        return value.Float;
    }

    if ("Text" in value) {
        return value.Text;
    }

    return value.Blob;
}

function normalizeRows(rows: SqlQueryResult["rows"]): string[][] {
    if (!Array.isArray(rows)) {
        return [];
    }

    return rows.map((row) =>
        row.map((cell) => fromSqlValue(cell as SqlValue)?.toString() ?? ""),
    );
}

function toSqlParam(value: unknown): SqlParam {
    if (value === null || value === undefined) {
        return "Null";
    }

    if (typeof value === "boolean") {
        return { Bool: value };
    }

    if (typeof value === "number") {
        return Number.isInteger(value) ? { Int: value } : { Float: value };
    }

    return { Text: String(value) };
}

export const db = drizzle(
    async (sql, params, method) => {
        const result = await backend.sql(
            sql,
            ((params as unknown[] | undefined) ?? []).map(toSqlParam),
            method,
        );

        const normalizedRows = normalizeRows(result.rows);

        if (method === "get") {
            return { rows: normalizedRows[0] ?? undefined };
        }

        return { rows: normalizedRows };
    },
    {
        relations,
    },
);
