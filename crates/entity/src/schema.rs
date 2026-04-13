// @generated automatically by Diesel CLI.

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
        timeslot_restriction_mode -> Nullable<Text>,
    }
}

diesel::table! {
    exam_constraint (exam1_id, exam2_id) {
        exam1_id -> Integer,
        exam2_id -> Integer,
        constraint_type -> Text,
    }
}

diesel::table! {
    exam_timeslot_restriction (exam_id, timeslot_id) {
        exam_id -> Integer,
        timeslot_id -> Integer,
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
    session_time_config (slot) {
        slot -> Integer,
        reading_start_time -> Time,
        exam_start_time -> Time,
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

diesel::table! {
    timetable_slots (timetable_id, session_id) {
        timetable_id -> Integer,
        session_id -> Integer,
        timeslot_id -> Integer,
        locked -> Bool,
        reading_start_time -> Nullable<Time>,
        exam_start_time -> Nullable<Time>,
        exam_end_time -> Nullable<Time>,
    }
}

diesel::table! {
    timetables (id) {
        id -> Integer,
        name -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::joinable!(enrolled_student -> student (student_id));
diesel::joinable!(enrolled_student -> subject (subject_id));
diesel::joinable!(exam -> subject (subject_id));
diesel::joinable!(exam_timeslot_restriction -> exam (exam_id));
diesel::joinable!(exam_timeslot_restriction -> timeslot (timeslot_id));
diesel::joinable!(session -> exam (exam_id));
diesel::joinable!(subject_grade -> subject (subject_id));
diesel::joinable!(timetable_slots -> session (session_id));
diesel::joinable!(timetable_slots -> timeslot (timeslot_id));
diesel::joinable!(timetable_slots -> timetables (timetable_id));

diesel::allow_tables_to_appear_in_same_query!(
    enrolled_student,
    exam,
    exam_constraint,
    exam_timeslot_restriction,
    session,
    session_time_config,
    student,
    subject,
    subject_grade,
    timeslot,
    timetable_slots,
    timetables,
);
