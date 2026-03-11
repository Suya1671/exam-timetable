-- Add migration script here

CREATE TABLE IF NOT EXISTS exam_allowed_timeslots (
    exam_id     INTEGER NOT NULL,
    timeslot_id INTEGER NOT NULL,
    PRIMARY KEY (exam_id, timeslot_id),
    FOREIGN KEY (exam_id)     REFERENCES exams(id)     ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (timeslot_id) REFERENCES timeslots(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS exam_denied_timeslots (
    exam_id INTEGER NOT NULL,
    timeslot_id INTEGER NOT NULL,
    PRIMARY KEY (exam_id, timeslot_id),
    FOREIGN KEY (exam_id) REFERENCES exams(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (timeslot_id) REFERENCES timeslots(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

-- Students
CREATE TABLE IF NOT EXISTS student_allowed_timeslots (
    student_id  INTEGER NOT NULL,
    timeslot_id INTEGER NOT NULL,
    PRIMARY KEY (student_id, timeslot_id),
    FOREIGN KEY (student_id)  REFERENCES students(id)  ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (timeslot_id) REFERENCES timeslots(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS student_denied_timeslots (
    student_id INTEGER NOT NULL,
    timeslot_id INTEGER NOT NULL,
    PRIMARY KEY (student_id, timeslot_id),
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (timeslot_id) REFERENCES timeslots(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

-- Subjects
CREATE TABLE IF NOT EXISTS subject_allowed_timeslots (
    subject_id INTEGER NOT NULL,
    timeslot_id INTEGER NOT NULL,
    PRIMARY KEY (subject_id, timeslot_id),
    FOREIGN KEY (subject_id) REFERENCES subjects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (timeslot_id) REFERENCES timeslots(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;

CREATE TABLE IF NOT EXISTS subject_denied_timeslots (
    subject_id INTEGER NOT NULL,
    timeslot_id INTEGER NOT NULL,
    PRIMARY KEY (subject_id, timeslot_id),
    FOREIGN KEY (subject_id) REFERENCES subjects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (timeslot_id) REFERENCES timeslots(id) ON DELETE CASCADE ON UPDATE CASCADE
) STRICT;
