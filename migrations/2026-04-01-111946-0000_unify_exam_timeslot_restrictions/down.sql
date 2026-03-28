CREATE TABLE exam_allowed_timeslot (
  exam_id INTEGER NOT NULL,
  timeslot_id INTEGER NOT NULL,
  PRIMARY KEY (exam_id, timeslot_id),
  FOREIGN KEY (exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (timeslot_id) REFERENCES timeslot (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE exam_denied_timeslot (
  exam_id INTEGER NOT NULL,
  timeslot_id INTEGER NOT NULL,
  PRIMARY KEY (exam_id, timeslot_id),
  FOREIGN KEY (exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (timeslot_id) REFERENCES timeslot (id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO exam_allowed_timeslot (exam_id, timeslot_id)
SELECT exam_id, timeslot_id
FROM exam_timeslot_restriction
WHERE exam_id IN (
  SELECT id
  FROM exam
  WHERE timeslot_restriction_mode = 'allow'
);

INSERT INTO exam_denied_timeslot (exam_id, timeslot_id)
SELECT exam_id, timeslot_id
FROM exam_timeslot_restriction
WHERE exam_id IN (
  SELECT id
  FROM exam
  WHERE timeslot_restriction_mode = 'deny'
);

DROP TABLE exam_timeslot_restriction;

ALTER TABLE exam
DROP COLUMN timeslot_restriction_mode;
