CREATE TABLE student (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  grade INTEGER NOT NULL
);

CREATE TABLE subject (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE subject_grade (
  subject_id INTEGER NOT NULL,
  grade INTEGER NOT NULL,
  PRIMARY KEY (subject_id, grade),
  FOREIGN KEY (subject_id) REFERENCES subject (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE timeslot (
  id INTEGER NOT NULL PRIMARY KEY,
  date TEXT NOT NULL,
  slot INTEGER NOT NULL
);

CREATE TABLE enrolled_student (
  student_id INTEGER NOT NULL,
  subject_id INTEGER NOT NULL,
  PRIMARY KEY (student_id, subject_id),
  FOREIGN KEY (student_id) REFERENCES student (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (subject_id) REFERENCES subject (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE exam (
  id INTEGER NOT NULL PRIMARY KEY,
  subject_id INTEGER NOT NULL,
  grade INTEGER NOT NULL,
  paper INTEGER NOT NULL,
  duration_hours real_decimal NOT NULL,
  priority INTEGER NOT NULL,
  slots_required INTEGER NOT NULL,
  UNIQUE (subject_id, grade, paper),
  FOREIGN KEY (subject_id) REFERENCES subject (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (subject_id, grade) REFERENCES subject_grade (subject_id, grade) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE exam_allowed_timeslot (
  exam_id INTEGER NOT NULL,
  timeslot_id INTEGER NOT NULL,
  PRIMARY KEY (exam_id, timeslot_id),
  FOREIGN KEY (exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (timeslot_id) REFERENCES timeslot (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE same_day_exam (
  first_slot_exam_id INTEGER NOT NULL,
  second_slot_exam_id INTEGER NOT NULL,
  PRIMARY KEY (first_slot_exam_id, second_slot_exam_id),
  FOREIGN KEY (first_slot_exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (second_slot_exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE exam_denied_timeslot (
  exam_id INTEGER NOT NULL,
  timeslot_id INTEGER NOT NULL,
  PRIMARY KEY (exam_id, timeslot_id),
  FOREIGN KEY (exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (timeslot_id) REFERENCES timeslot (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE session (
  id INTEGER NOT NULL PRIMARY KEY,
  exam_id INTEGER NOT NULL,
  sequence INTEGER NOT NULL,
  FOREIGN KEY (exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE different_week_exams (
  exam1_id INTEGER NOT NULL,
  exam2_id INTEGER NOT NULL,
  PRIMARY KEY (exam1_id, exam2_id),
  FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TRIGGER check_student_subject_grade BEFORE INSERT ON enrolled_student FOR EACH ROW BEGIN
SELECT
  CASE
    WHEN NOT EXISTS (
      SELECT
        1
      FROM
        student s
        JOIN subject_grade sg ON sg.subject_id = NEW.subject_id
        AND sg.grade = s.grade
      WHERE
        s.id = NEW.student_id
    ) THEN RAISE (ABORT, 'Student grade not allowed for subject')
  END;

END;
