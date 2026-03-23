import { defineRelations } from "drizzle-orm";
import * as schema from "./schema";

export const relations = defineRelations(schema, (r) => ({
    subjectGrade: {
        subject: r.one.subject({
            from: r.subjectGrade.subjectId,
            to: r.subject.id,
            alias: "subjectGrade_subjectId_subject_id",
        }),
        subjects: r.many.subject({
            from: [
                r.subjectGrade.subjectId.through(r.exam.subjectId),
                r.subjectGrade.grade.through(r.exam.grade),
            ],
            to: r.subject.id.through(r.exam.subjectId),
            alias: "subjectGrade_subjectId_grade_subject_id_via_exam",
        }),
        exams: r.many.exam({
            from: [r.subjectGrade.subjectId, r.subjectGrade.grade],
            to: [r.exam.subjectId, r.exam.grade],
        }),
    },
    subject: {
        subjectGradesSubjectId: r.many.subjectGrade({
            alias: "subjectGrade_subjectId_subject_id",
        }),
        students: r.many.student({
            from: r.subject.id.through(r.enrolledStudent.subjectId),
            to: r.student.id.through(r.enrolledStudent.studentId),
        }),
        subjectGradesViaExam: r.many.subjectGrade({
            alias: "subjectGrade_subjectId_grade_subject_id_via_exam",
        }),
    },
    student: {
        subjects: r.many.subject(),
    },
    timeslot: {
        examsViaExamAllowedTimeslot: r.many.exam({
            from: r.timeslot.id.through(r.examAllowedTimeslot.timeslotId),
            to: r.exam.id.through(r.examAllowedTimeslot.examId),
            alias: "timeslot_id_exam_id_via_examAllowedTimeslot",
        }),
        examsViaExamDeniedTimeslot: r.many.exam({
            from: r.timeslot.id.through(r.examDeniedTimeslot.timeslotId),
            to: r.exam.id.through(r.examDeniedTimeslot.examId),
            alias: "timeslot_id_exam_id_via_examDeniedTimeslot",
        }),
    },
    exam: {
        timeslotsViaExamAllowedTimeslot: r.many.timeslot({
            alias: "timeslot_id_exam_id_via_examAllowedTimeslot",
        }),
        timeslotsViaExamDeniedTimeslot: r.many.timeslot({
            alias: "timeslot_id_exam_id_via_examDeniedTimeslot",
        }),
        sessions: r.many.session(),
        subject: r.one.subject({
            from: r.exam.subjectId,
            to: r.subject.id,
        }),
    },
    session: {
        exam: r.one.exam({
            from: r.session.examId,
            to: r.exam.id,
        }),
    },
}));
