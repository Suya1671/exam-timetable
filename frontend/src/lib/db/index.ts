import {
	createTauRPCProxy,
	type SqlError,
	type SqlParam,
	type SqlQueryResult,
	type SqlValue
} from '$lib/backend';
import { drizzle } from 'drizzle-orm/sqlite-proxy';

import { relations } from './relations';

const backend = createTauRPCProxy() as unknown as {
	sql: (sql: string, params: SqlParam[], method: string) => Promise<SqlQueryResult>;
	sql_batch: (
		queries: { sql: string; params: SqlParam[]; method: string }[]
	) => Promise<SqlQueryResult[]>;
};

function fromSqlValue(value: SqlValue) {
	if (value === 'Null') {
		return null;
	}

	if ('Bool' in value) {
		return value.Bool;
	}

	if ('Int' in value) {
		return value.Int;
	}

	if ('Float' in value) {
		return value.Float;
	}

	if ('Text' in value) {
		return value.Text;
	}

	return value.Blob;
}

function normalizeRows(rows: SqlQueryResult['rows']) {
	if (!Array.isArray(rows)) {
		return [];
	}

	return rows.map((row) => row.map((cell) => fromSqlValue(cell as SqlValue)));
}

function toSqlParam(value: unknown): SqlParam {
	if (value === null || value === undefined) {
		return 'Null';
	}

	if (typeof value === 'boolean') {
		return { Bool: value };
	}

	if (typeof value === 'number') {
		return Number.isInteger(value) ? { Int: value } : { Float: value };
	}

	return { Text: String(value) };
}

/** AI-generated (GPT-5.3-codex). */
function formatSqlError(error: unknown): string {
	if (!error || typeof error !== 'object') {
		return 'Unknown SQL RPC error';
	}

	const typed = error as SqlError;
	if ('SqlProxy' in typed) {
		return typed.SqlProxy;
	}

	if ('LockPoison' in typed) {
		return typed.LockPoison;
	}

	return 'Unknown SQL RPC error';
}

export const db = drizzle(
	async (sql, params, method) => {
		let result: SqlQueryResult;
		try {
			result = await backend.sql(
				sql,
				((params as unknown[] | undefined) ?? []).map(toSqlParam),
				method
			);
		} catch (error) {
			throw new Error(`SQL RPC failed: ${formatSqlError(error)}`, {
				cause: error
			});
		}

		const normalizedRows = normalizeRows(result.rows);

		if (method === 'get') {
			return { rows: normalizedRows[0] ?? [] };
		}

		return { rows: normalizedRows };
	},
	async (queries) => {
		let result: SqlQueryResult[];
		try {
			result = await backend.sql_batch(
				queries.map(({ sql, params, method }) => ({
					sql,
					params: ((params as unknown[] | undefined) ?? []).map(toSqlParam),
					method
				}))
			);
		} catch (error) {
			throw new Error(`SQL batch RPC failed: ${formatSqlError(error)}`, {
				cause: error
			});
		}

		return result.map((queryResult, index) => {
			const method = queries[index]?.method;
			const normalizedRows = normalizeRows(queryResult.rows);

			if (method === 'get') {
				return { rows: normalizedRows[0] ?? [] };
			}

			return { rows: normalizedRows };
		});
	},
	{
		relations
	}
);
