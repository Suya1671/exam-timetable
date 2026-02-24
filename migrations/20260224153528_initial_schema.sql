-- Add migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS subjects (
    id        INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name      TEXT    NOT NULL,
    grade     INTEGER NOT NULL CHECK (grade BETWEEN 1 AND 12),

    -- A subject name can appear only once per grade (e.g. “Math” in Grade 12)
    UNIQUE (name, grade)
) STRICT;

CREATE TABLE IF NOT EXISTS exams (
    id               INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    subject_id       INTEGER NOT NULL,
    paper            INTEGER NOT NULL CHECK (paper >= 1),
    duration_hours   REAL    NOT NULL CHECK (duration_hours > 0),
    priority         INTEGER NOT NULL CHECK (priority BETWEEN 0 AND 255),

    -- Note: ensure UI says that deletion will delete associated exams
    FOREIGN KEY (subject_id) REFERENCES subjects(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS students (
    id    INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name  TEXT    NOT NULL,
    grade INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS enrolled_students (
    student_id INTEGER NOT NULL,
    subject_id INTEGER NOT NULL,

    PRIMARY KEY (student_id, subject_id),

    -- Note: ensure UI says that deletion will delete associated enrollments
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (subject_id) REFERENCES subjects(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS timeslots (
    id   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    date TEXT    NOT NULL, -- https://docs.rs/sqlx/latest/sqlx/sqlite/types/index.html#time
    slot INTEGER NOT NULL CHECK (slot IN (0, 1)),   -- 0 = First, 1 = Second

    -- A day cannot have 2 of the same slot (that just. doesn't make sense)
    UNIQUE (date, slot)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_exams_subject_id   ON exams   (subject_id);
CREATE INDEX IF NOT EXISTS idx_enrolled_student   ON enrolled_students (student_id);
CREATE INDEX IF NOT EXISTS idx_enrolled_subject   ON enrolled_students (subject_id);
CREATE INDEX IF NOT EXISTS idx_timeslots_date_slot ON timeslots (date, slot);
