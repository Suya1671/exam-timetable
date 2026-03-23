import {
    sqliteTable,
    foreignKey,
    primaryKey,
    text,
    integer,
    customType,
    real,
} from "drizzle-orm/sqlite-core";
import { customDate } from "./customTypes";

export const dieselSchemaMigrations = sqliteTable(
    "__diesel_schema_migrations",
    {
        version: text().primaryKey(),
        runOn: customType({ dataType: () => "TIMESTAMP" })("run_on")
            .default("CURRENT_TIMESTAMP")
            .notNull(),
    },
);

export const student = sqliteTable("student", {
    id: integer().primaryKey(),
    name: text().notNull(),
    grade: integer().notNull(),
});

export const subject = sqliteTable("subject", {
    id: integer().primaryKey(),
    name: text().notNull(),
});

export const subjectGrade = sqliteTable(
    "subject_grade",
    {
        subjectId: integer("subject_id")
            .notNull()
            .references(() => subject.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        grade: integer().notNull(),
    },
    (table) => [
        primaryKey({
            columns: [table.subjectId, table.grade],
            name: "subject_grade_pk",
        }),
    ],
);

export const timeslot = sqliteTable("timeslot", {
    id: integer().primaryKey(),
    date: customDate().notNull(),
    slot: integer().notNull(),
});

export const enrolledStudent = sqliteTable(
    "enrolled_student",
    {
        studentId: integer("student_id")
            .notNull()
            .references(() => student.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        subjectId: integer("subject_id")
            .notNull()
            .references(() => subject.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
    },
    (table) => [
        primaryKey({
            columns: [table.studentId, table.subjectId],
            name: "enrolled_student_pk",
        }),
    ],
);

export const exam = sqliteTable(
    "exam",
    {
        id: integer().primaryKey(),
        subjectId: integer("subject_id")
            .notNull()
            .references(() => subject.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        grade: integer().notNull(),
        paper: integer().notNull(),
        durationHours: real("duration_hours").notNull(),
        priority: integer().notNull(),
        slotsRequired: integer("slots_required").notNull(),
    },
    (table) => [
        foreignKey({
            columns: [table.subjectId, table.grade],
            foreignColumns: [subjectGrade.subjectId, subjectGrade.grade],
            name: "fk_exam_subject_id_grade_subject_grade_subject_id_grade_fk",
        })
            .onUpdate("cascade")
            .onDelete("cascade"),
    ],
);

export const examAllowedTimeslot = sqliteTable(
    "exam_allowed_timeslot",
    {
        examId: integer("exam_id")
            .notNull()
            .references(() => exam.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        timeslotId: integer("timeslot_id")
            .notNull()
            .references(() => timeslot.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
    },
    (table) => [
        primaryKey({
            columns: [table.examId, table.timeslotId],
            name: "exam_allowed_timeslot_pk",
        }),
    ],
);

export const sameDayExam = sqliteTable(
    "same_day_exam",
    {
        firstSlotExamId: integer("first_slot_exam_id")
            .notNull()
            .references(() => exam.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        secondSlotExamId: integer("second_slot_exam_id")
            .notNull()
            .references(() => exam.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        date: customDate().notNull(),
    },
    (table) => [
        primaryKey({
            columns: [table.firstSlotExamId, table.secondSlotExamId],
            name: "same_day_exam_pk",
        }),
    ],
);

export const examDeniedTimeslot = sqliteTable(
    "exam_denied_timeslot",
    {
        examId: integer("exam_id")
            .notNull()
            .references(() => exam.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        timeslotId: integer("timeslot_id")
            .notNull()
            .references(() => timeslot.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
    },
    (table) => [
        primaryKey({
            columns: [table.examId, table.timeslotId],
            name: "exam_denied_timeslot_pk",
        }),
    ],
);

export const session = sqliteTable("session", {
    id: integer().primaryKey(),
    examId: integer("exam_id")
        .notNull()
        .references(() => exam.id, {
            onDelete: "cascade",
            onUpdate: "cascade",
        }),
    sequence: integer().notNull(),
});

export const differentWeekExams = sqliteTable(
    "different_week_exams",
    {
        exam1Id: integer("exam1_id")
            .notNull()
            .references(() => exam.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
        exam2Id: integer("exam2_id")
            .notNull()
            .references(() => exam.id, {
                onDelete: "cascade",
                onUpdate: "cascade",
            }),
    },
    (table) => [
        primaryKey({
            columns: [table.exam1Id, table.exam2Id],
            name: "different_week_exams_pk",
        }),
    ],
);
