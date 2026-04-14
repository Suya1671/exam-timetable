CREATE TABLE exam_new (
  id INTEGER NOT NULL PRIMARY KEY,
  subject_id INTEGER NOT NULL,
  grade INTEGER NOT NULL,
  paper INTEGER NOT NULL,
  duration_hours real_decimal NOT NULL,
  priority INTEGER NOT NULL,
  slots_required INTEGER NOT NULL,
  timeslot_restriction_mode TEXT CHECK (timeslot_restriction_mode IN ('allow', 'deny')),
  UNIQUE (subject_id, grade, paper),
  FOREIGN KEY (subject_id) REFERENCES subject (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (subject_id, grade) REFERENCES subject_grade (subject_id, grade) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO exam_new (id, subject_id, grade, paper, duration_hours, priority, slots_required, timeslot_restriction_mode)
SELECT id, subject_id, grade, paper, duration_hours, priority, slots_required, times
lot_restriction_mode
FROM exam;

DROP TABLE exam;

ALTER TABLE exam_new RENAME TO exam;