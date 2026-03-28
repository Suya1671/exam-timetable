CREATE UNIQUE INDEX session_exam_id_sequence_uq ON session (exam_id, sequence);

DELETE FROM session;

WITH RECURSIVE seq(value) AS (
  SELECT 0
  UNION ALL
  SELECT value + 1
  FROM seq
  WHERE value + 1 < (SELECT COALESCE(MAX(slots_required), 0) FROM exam)
)
INSERT INTO session (exam_id, sequence)
SELECT
  exam.id,
  seq.value
FROM exam
JOIN seq ON seq.value < exam.slots_required;

CREATE TRIGGER session_create_after_exam_insert
AFTER INSERT ON exam
BEGIN
  INSERT INTO session (exam_id, sequence)
  WITH RECURSIVE seq(value) AS (
    SELECT 0
    UNION ALL
    SELECT value + 1
    FROM seq
    WHERE value + 1 < NEW.slots_required
  )
  SELECT
    NEW.id,
    seq.value
  FROM seq
  WHERE NEW.slots_required > 0;
END;

CREATE TRIGGER session_rebuild_after_slots_required_update
AFTER UPDATE OF slots_required ON exam
WHEN OLD.slots_required <> NEW.slots_required
BEGIN
  DELETE FROM session
  WHERE exam_id = NEW.id;

  INSERT INTO session (exam_id, sequence)
  WITH RECURSIVE seq(value) AS (
    SELECT 0
    UNION ALL
    SELECT value + 1
    FROM seq
    WHERE value + 1 < NEW.slots_required
  )
  SELECT
    NEW.id,
    seq.value
  FROM seq
  WHERE NEW.slots_required > 0;
END;
