CREATE TABLE timetables (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE timetable_slots (
  timetable_id INTEGER NOT NULL,
  session_id INTEGER NOT NULL,
  timeslot_id INTEGER NOT NULL,
  locked BOOLEAN NOT NULL DEFAULT 0,
  reading_start_time TEXT,
  exam_start_time TEXT,
  exam_end_time TEXT,
  PRIMARY KEY (timetable_id, session_id),
  FOREIGN KEY (timetable_id) REFERENCES timetables (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (session_id) REFERENCES session (id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (timeslot_id) REFERENCES timeslot (id) ON DELETE CASCADE ON UPDATE CASCADE
);
