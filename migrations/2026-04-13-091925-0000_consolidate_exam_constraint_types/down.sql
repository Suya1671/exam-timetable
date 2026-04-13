-- Undo consolidation: recreate old tables from exam_constraint data

-- Restore same_day_exam (same_day type)
CREATE TABLE same_day_exam (
    first_slot_exam_id INTEGER NOT NULL,
    second_slot_exam_id INTEGER NOT NULL,
    PRIMARY KEY (first_slot_exam_id, second_slot_exam_id),
    FOREIGN KEY (first_slot_exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (second_slot_exam_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO same_day_exam (first_slot_exam_id, second_slot_exam_id)
SELECT exam1_id, exam2_id FROM exam_constraint WHERE constraint_type = 'same_day';

-- Restore different_week_exams
CREATE TABLE different_week_exams (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    PRIMARY KEY (exam1_id, exam2_id),
    FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO different_week_exams (exam1_id, exam2_id)
SELECT exam1_id, exam2_id FROM exam_constraint WHERE constraint_type = 'different_week';

-- Restore same_time_exam
CREATE TABLE same_time_exam (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    PRIMARY KEY (exam1_id, exam2_id),
    FOREIGN KEY (exam1_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exam (id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO same_time_exam (exam1_id, exam2_id)
SELECT exam1_id, exam2_id FROM exam_constraint WHERE constraint_type = 'same_time';

-- Drop the consolidated table
DROP TABLE IF EXISTS exam_constraint;
