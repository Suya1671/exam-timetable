import { defineRelations } from 'drizzle-orm';
import * as schema from './schema';

export const relations = defineRelations(schema, (r) => ({
	subjectGrade: {
		subject: r.one.subject({
			from: r.subjectGrade.subjectId,
			to: r.subject.id,
			alias: 'subjectGrade_subjectId_subject_id',
			optional: false
		}),
		subjects: r.many.subject({
			from: [
				r.subjectGrade.subjectId.through(r.exam.subjectId),
				r.subjectGrade.grade.through(r.exam.grade)
			],
			to: r.subject.id.through(r.exam.subjectId),
			alias: 'subjectGrade_subjectId_grade_subject_id_via_exam'
		}),
		exams: r.many.exam({
			from: [r.subjectGrade.subjectId, r.subjectGrade.grade],
			to: [r.exam.subjectId, r.exam.grade]
		})
	},
	subject: {
		subjectGradesSubjectId: r.many.subjectGrade({
			alias: 'subjectGrade_subjectId_subject_id'
		}),
		students: r.many.student({
			from: r.subject.id.through(r.enrolledStudent.subjectId),
			to: r.student.id.through(r.enrolledStudent.studentId)
		}),
		subjectGradesViaExam: r.many.subjectGrade({
			alias: 'subjectGrade_subjectId_grade_subject_id_via_exam'
		})
	},
	student: {
		subjects: r.many.subject()
	},
	timeslot: {
		examsRestrictedBy: r.many.exam({
			from: r.timeslot.id.through(r.examTimeslotRestriction.timeslotId),
			to: r.exam.id.through(r.examTimeslotRestriction.examId),
			alias: 'timeslot_id_exam_id_via_examTimeslotRestriction'
		}),
		timetableSlots: r.many.timetableSlots()
	},
	exam: {
		restrictedTimeslots: r.many.timeslot({
			alias: 'timeslot_id_exam_id_via_examTimeslotRestriction'
		}),
		sessions: r.many.session(),
		subject: r.one.subject({
			from: r.exam.subjectId,
			to: r.subject.id,
			optional: false
		})
	},
	examTimeConstraint: {
		firstExam: r.one.exam({
			from: r.examTimeConstraint.exam1Id,
			to: r.exam.id,
			optional: false
		}),
		secondExam: r.one.exam({
			from: r.examTimeConstraint.exam2Id,
			to: r.exam.id,
			optional: false
		})
	},
	examOrderConstraint: {
		firstExam: r.one.exam({
			from: r.examOrderConstraint.exam1Id,
			to: r.exam.id,
			optional: false
		}),
		secondExam: r.one.exam({
			from: r.examOrderConstraint.exam2Id,
			to: r.exam.id,
			optional: false
		})
	},
	session: {
		exam: r.one.exam({
			from: r.session.examId,
			to: r.exam.id,
			optional: false
		}),
		timetableSlots: r.many.timetableSlots()
	},
	timetables: {
		slots: r.many.timetableSlots()
	},
	timetableSlots: {
		timetable: r.one.timetables({
			from: r.timetableSlots.timetableId,
			to: r.timetables.id,
			optional: false
		}),
		session: r.one.session({
			from: r.timetableSlots.sessionId,
			to: r.session.id,
			optional: false
		}),
		timeslot: r.one.timeslot({
			from: r.timetableSlots.timeslotId,
			to: r.timeslot.id,
			optional: false
		})
	}
}));
