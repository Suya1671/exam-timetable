-- Add migration script here
ALTER TABLE exams
ADD COLUMN slots_required INTEGER NOT NULL DEFAULT 1; -- consecutive slots required for multi-day exams
