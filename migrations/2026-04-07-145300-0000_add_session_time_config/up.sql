CREATE TABLE session_time_config (
  slot INTEGER NOT NULL PRIMARY KEY,
  reading_start_time TEXT NOT NULL,
  exam_start_time TEXT NOT NULL
);

INSERT INTO session_time_config (slot, reading_start_time, exam_start_time)
VALUES
  (0, '07:45:00', '08:00:00'),
  (1, '11:45:00', '12:00:00');
