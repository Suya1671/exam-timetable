import { defineConfig } from 'drizzle-kit';

export default defineConfig({
	dialect: 'sqlite',
	out: './src/lib/db',
	dbCredentials: {
		url: '../db.sqlite3'
	},
	verbose: true,
	breakpoints: true
});
