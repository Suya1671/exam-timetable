-- Add migration script here
CREATE TABLE IF NOT EXISTS different_week_exams (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    FOREIGN KEY (exam1_id) REFERENCES exams(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exams(id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (exam1_id, exam2_id)
);
