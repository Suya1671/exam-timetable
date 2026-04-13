import {
	sqliteTable,
	foreignKey,
	primaryKey,
	text,
	integer,
	customType,
	real
} from 'drizzle-orm/sqlite-core';
import { customDate, customTime } from './customTypes';

export const dieselSchemaMigrations = sqliteTable('__diesel_schema_migrations', {
	version: text().primaryKey(),
	runOn: customType({ dataType: () => 'TIMESTAMP' })('run_on')
		.default('CURRENT_TIMESTAMP')
		.notNull()
});

export const student = sqliteTable('student', {
	id: integer().primaryKey(),
	name: text().notNull(),
	grade: integer().notNull()
});

export const subject = sqliteTable('subject', {
	id: integer().primaryKey(),
	name: text().notNull()
});

export const subjectGrade = sqliteTable(
	'subject_grade',
	{
		subjectId: integer('subject_id')
			.notNull()
			.references(() => subject.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		grade: integer().notNull()
	},
	(table) => [
		primaryKey({
			columns: [table.subjectId, table.grade],
			name: 'subject_grade_pk'
		})
	]
);

export const timeslot = sqliteTable('timeslot', {
	id: integer().primaryKey(),
	date: customDate().notNull(),
	slot: integer().notNull()
});

export const enrolledStudent = sqliteTable(
	'enrolled_student',
	{
		studentId: integer('student_id')
			.notNull()
			.references(() => student.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		subjectId: integer('subject_id')
			.notNull()
			.references(() => subject.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			})
	},
	(table) => [
		primaryKey({
			columns: [table.studentId, table.subjectId],
			name: 'enrolled_student_pk'
		})
	]
);

export const exam = sqliteTable(
	'exam',
	{
		id: integer().primaryKey(),
		subjectId: integer('subject_id')
			.notNull()
			.references(() => subject.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		grade: integer().notNull(),
		paper: integer().notNull(),
		durationHours: real('duration_hours').notNull(),
		priority: integer().notNull(),
		slotsRequired: integer('slots_required').notNull(),
		timeslotRestrictionMode: text('timeslot_restriction_mode', {
			enum: ['allow', 'deny']
		})
	},
	(table) => [
		foreignKey({
			columns: [table.subjectId, table.grade],
			foreignColumns: [subjectGrade.subjectId, subjectGrade.grade],
			name: 'fk_exam_subject_id_grade_subject_grade_subject_id_grade_fk'
		})
			.onUpdate('cascade')
			.onDelete('cascade')
	]
);

export const examConstraint = sqliteTable(
	'exam_constraint',
	{
		exam1Id: integer('exam1_id')
			.notNull()
			.references(() => exam.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		exam2Id: integer('exam2_id')
			.notNull()
			.references(() => exam.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		constraintType: text('constraint_type', {
			enum: [
				'same_day',
				'different_day',
				'same_week',
				'different_week',
				'same_time',
				'different_time'
			]
		}).notNull()
	},
	(table) => [
		primaryKey({
			columns: [table.exam1Id, table.exam2Id, table.constraintType],
			name: 'exam_constraint_pk'
		})
	]
);

export type ExamConstraintType = typeof examConstraint.$inferSelect.constraintType;

export const examTimeslotRestriction = sqliteTable(
	'exam_timeslot_restriction',
	{
		examId: integer('exam_id')
			.notNull()
			.references(() => exam.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		timeslotId: integer('timeslot_id')
			.notNull()
			.references(() => timeslot.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			})
	},
	(table) => [
		primaryKey({
			columns: [table.examId, table.timeslotId],
			name: 'exam_timeslot_restriction_pk'
		})
	]
);

export const session = sqliteTable('session', {
	id: integer().primaryKey(),
	examId: integer('exam_id')
		.notNull()
		.references(() => exam.id, {
			onDelete: 'cascade',
			onUpdate: 'cascade'
		}),
	sequence: integer().notNull()
});

export const sessionTimeConfig = sqliteTable('session_time_config', {
	slot: integer().primaryKey(),
	readingStartTime: customTime('reading_start_time').notNull(),
	examStartTime: customTime('exam_start_time').notNull()
});

export const timetables = sqliteTable('timetables', {
	id: integer().primaryKey(),
	name: text().notNull(),
	createdAt: text('created_at').notNull(),
	updatedAt: text('updated_at').notNull()
});

export const timetableSlots = sqliteTable(
	'timetable_slots',
	{
		timetableId: integer('timetable_id')
			.notNull()
			.references(() => timetables.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		sessionId: integer('session_id')
			.notNull()
			.references(() => session.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		timeslotId: integer('timeslot_id')
			.notNull()
			.references(() => timeslot.id, {
				onDelete: 'cascade',
				onUpdate: 'cascade'
			}),
		locked: integer({ mode: 'boolean' }).notNull().default(false),
		readingStartTime: customTime('reading_start_time'),
		examStartTime: customTime('exam_start_time'),
		examEndTime: customTime('exam_end_time')
	},
	(table) => [
		primaryKey({
			columns: [table.timetableId, table.sessionId],
			name: 'timetable_slots_pk'
		})
	]
);
