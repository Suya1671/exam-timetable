PRAGMA foreign_keys = OFF;

-- Drop the ordering constraints table (this loses the 'before' constraints)
DROP TABLE IF EXISTS exam_order_constraint;

-- Rename time constraints back to exam_constraint
ALTER TABLE exam_time_constraint RENAME TO exam_constraint;

-- Restore the CHECK constraint with all constraint types including 'before'
CREATE TABLE exam_constraint_new (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    constraint_type TEXT NOT NULL,
    PRIMARY KEY (exam1_id, exam2_id),
    FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CHECK (constraint_type IN ('same_time', 'different_time', 'same_day', 'different_day', 'same_week', 'different_week', 'before'))
);

INSERT INTO exam_constraint_new (exam1_id, exam2_id, constraint_type)
SELECT exam1_id, exam2_id, constraint_type FROM exam_constraint;

DROP TABLE exam_constraint;
ALTER TABLE exam_constraint_new RENAME TO exam_constraint;

PRAGMA foreign_keys = ON;