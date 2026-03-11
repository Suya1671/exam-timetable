-- Add migration script here
CREATE TABLE IF NOT EXISTS same_day_exams (
    exam1_id INTEGER NOT NULL,
    exam2_id INTEGER NOT NULL,
    date     TEXT    NOT NULL, -- https://docs.rs/sqlx/latest/sqlx/sqlite/types/index.html#time
    FOREIGN KEY (exam1_id) REFERENCES exams(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (exam2_id) REFERENCES exams(id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (exam1_id, exam2_id)
)
