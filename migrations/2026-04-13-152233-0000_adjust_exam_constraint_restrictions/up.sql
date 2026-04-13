PRAGMA foreign_keys = OFF;

-- Create separate table for ordering constraints
CREATE TABLE exam_order_constraint (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    PRIMARY KEY (exam1_id, exam2_id),
    FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

-- Migrate existing 'before' constraints
INSERT INTO exam_order_constraint (exam1_id, exam2_id)
SELECT exam1_id, exam2_id FROM exam_constraint WHERE constraint_type = 'before';

-- Drop the 'before' constraint type from exam_constraint
DELETE FROM exam_constraint WHERE constraint_type = 'before';

CREATE TABLE exam_time_constraint (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    constraint_type TEXT NOT NULL,
    PRIMARY KEY (exam1_id, exam2_id),
    FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CHECK (constraint_type IN ('same_time', 'different_time', 'same_day', 'different_day', 'same_week', 'different_week'))
);

INSERT INTO exam_time_constraint (exam1_id, exam2_id, constraint_type)
SELECT exam1_id, exam2_id, constraint_type FROM exam_constraint;

DROP TABLE exam_constraint;

PRAGMA foreign_keys = ON;
