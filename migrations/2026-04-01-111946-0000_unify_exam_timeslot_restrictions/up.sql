ALTER TABLE exam
ADD COLUMN timeslot_restriction_mode TEXT
CHECK (timeslot_restriction_mode IN ('allow', 'deny'));

CREATE TABLE exam_timeslot_restriction (
  exam_id INTEGER NOT NULL,
  timeslot_id INTEGER NOT NULL,
  PRIMARY KEY (exam_id, timeslot_id),
  FOREIGN KEY (exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (timeslot_id) REFERENCES timeslot (id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO exam_timeslot_restriction (exam_id, timeslot_id)
SELECT a.exam_id, a.timeslot_id
FROM exam_allowed_timeslot a;

INSERT INTO exam_timeslot_restriction (exam_id, timeslot_id)
SELECT d.exam_id, d.timeslot_id
FROM exam_denied_timeslot d
WHERE d.exam_id NOT IN (SELECT exam_id FROM exam_allowed_timeslot);

UPDATE exam
SET timeslot_restriction_mode = 'allow'
WHERE id IN (SELECT exam_id FROM exam_allowed_timeslot);

UPDATE exam
SET timeslot_restriction_mode = 'deny'
WHERE id IN (SELECT exam_id FROM exam_denied_timeslot)
AND id NOT IN (SELECT exam_id FROM exam_allowed_timeslot);

DROP TABLE exam_allowed_timeslot;
DROP TABLE exam_denied_timeslot;
