-- Consolidate 2-day exam constraint types into a single table
-- Constraints: same_day, different_day, same_week, different_week, same_time

CREATE TABLE exam_constraint (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    constraint_type TEXT NOT NULL,
    PRIMARY KEY (exam1_id, exam2_id),
    FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CHECK (constraint_type IN ('same_time', 'different_time', 'same_day', 'different_day', 'same_week', 'different_week'))
);

-- Migrate existing same_day_exam (consecutive) constraints
INSERT INTO exam_constraint (exam1_id, exam2_id, constraint_type)
SELECT first_slot_exam_id, second_slot_exam_id, 'same_day'
FROM same_day_exam;

-- Migrate existing different_week_exams constraints
INSERT INTO exam_constraint (exam1_id, exam2_id, constraint_type)
SELECT exam1_id, exam2_id, 'different_week'
FROM different_week_exams;

-- Migrate existing same_time_exam constraints
INSERT INTO exam_constraint (exam1_id, exam2_id, constraint_type)
SELECT exam1_id, exam2_id, 'same_time'
FROM same_time_exam;

-- Drop old tables
DROP TABLE IF EXISTS same_day_exam;
DROP TABLE IF EXISTS different_week_exams;
DROP TABLE IF EXISTS same_time_exam;
