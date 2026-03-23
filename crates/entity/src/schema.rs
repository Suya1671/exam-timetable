// @generated automatically by Diesel CLI.

diesel::table! {
    different_week_exams (exam1_id, exam2_id) {
        exam1_id -> Integer,
        exam2_id -> Integer,
    }
}

diesel::table! {
    enrolled_student (student_id, subject_id) {
        student_id -> Integer,
        subject_id -> Integer,
    }
}

diesel::table! {
    exam (id) {
        id -> Integer,
        subject_id -> Integer,
        grade -> Integer,
        paper -> Integer,
        duration_hours -> Float,
        priority -> Integer,
        slots_required -> Integer,
    }
}

diesel::table! {
    exam_allowed_timeslot (exam_id, timeslot_id) {
        exam_id -> Integer,
        timeslot_id -> Integer,
    }
}

diesel::table! {
    exam_denied_timeslot (exam_id, timeslot_id) {
        exam_id -> Integer,
        timeslot_id -> Integer,
    }
}

diesel::table! {
    same_day_exam (first_slot_exam_id, second_slot_exam_id) {
        first_slot_exam_id -> Integer,
        second_slot_exam_id -> Integer,
        date -> Date,
    }
}

diesel::table! {
    session (id) {
        id -> Integer,
        exam_id -> Integer,
        sequence -> Integer,
    }
}

diesel::table! {
    student (id) {
        id -> Integer,
        name -> Text,
        grade -> Integer,
    }
}

diesel::table! {
    subject (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    subject_grade (subject_id, grade) {
        subject_id -> Integer,
        grade -> Integer,
    }
}

diesel::table! {
    timeslot (id) {
        id -> Integer,
        date -> Date,
        slot -> Integer,
    }
}

diesel::joinable!(enrolled_student -> student (student_id));
diesel::joinable!(enrolled_student -> subject (subject_id));
diesel::joinable!(exam -> subject (subject_id));
diesel::joinable!(exam_allowed_timeslot -> exam (exam_id));
diesel::joinable!(exam_allowed_timeslot -> timeslot (timeslot_id));
diesel::joinable!(exam_denied_timeslot -> exam (exam_id));
diesel::joinable!(exam_denied_timeslot -> timeslot (timeslot_id));
diesel::joinable!(session -> exam (exam_id));
diesel::joinable!(subject_grade -> subject (subject_id));

diesel::allow_tables_to_appear_in_same_query!(
    different_week_exams,
    enrolled_student,
    exam,
    exam_allowed_timeslot,
    exam_denied_timeslot,
    same_day_exam,
    session,
    student,
    subject,
    subject_grade,
    timeslot,
);
