<script lang="ts">
	import { db } from '$lib/db';
	import { exam, subject, subjectGrade } from '$lib/db/schema';
	import { and, eq } from 'drizzle-orm';
	import AddSubjectDialog from './AddSubjectDialog.svelte';
	import EditSubjectDialog from './EditSubjectDialog.svelte';
	import AddGradeDialog from './AddGradeDialog.svelte';
	import AddExamDialog from './AddExamDialog.svelte';
	import EditExamDialog from './EditExamDialog.svelte';
	import SubjectsSection from './SubjectsSection.svelte';
	import PlusIcon from '@ktibow/iconset-material-symbols/add-rounded';
	import { Button, Icon } from 'm3-svelte';

	/** AI-generated (GPT-5.3-codex). */
	async function getSubjects() {
		return db.query.subject.findMany({
			with: {
				subjectGradesSubjectId: {
					with: {
						exams: {
							orderBy: {
								paper: 'asc'
							}
						}
					},
					orderBy: {
						grade: 'asc'
					}
				}
			},
			orderBy: {
				name: 'asc'
			}
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getTimeslots() {
		return db.query.timeslot.findMany({
			orderBy: {
				date: 'asc',
				slot: 'asc'
			}
		});
	}

	let data = $state(await getSubjects());
	let allTimeslots = $state(await getTimeslots());

	type ActiveDialog =
		| { type: 'addSubject' }
		| { type: 'editSubject'; subjectId: number; name: string }
		| { type: 'addGrade'; subjectId: number }
		| { type: 'addExam'; subjectId: number; grade: number; paper: number }
		| { type: 'editExam'; exam: typeof exam.$inferSelect }
		| null;

	let activeDialog = $state<ActiveDialog>(null);

	/** AI-generated (GPT-5.3-codex). */
	async function refreshSubjects() {
		data = await getSubjects();
	}

	/** AI-generated (GPT-5.3-codex). */
	function closeDialog() {
		activeDialog = null;
	}

	/** AI-generated (GPT-5.3-codex). */
	async function deleteSubject(subjectId: number) {
		if (!confirm('Are you sure you want to delete this subject? This action cannot be undone.'))
			return;
		await db.delete(subject).where(eq(subject.id, subjectId));
		await refreshSubjects();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function deleteGrade(subjectId: number, grade: number) {
		if (!confirm('Are you sure you want to delete this grade? This action cannot be undone.'))
			return;
		await db
			.delete(subjectGrade)
			.where(and(eq(subjectGrade.subjectId, subjectId), eq(subjectGrade.grade, grade)));
		await refreshSubjects();
	}
</script>

<header>
	<h1>Subjects</h1>

	<menu aria-label="Subject actions">
		<li>
			<Button iconType="left" onclick={() => (activeDialog = { type: 'addSubject' })}>
				<Icon icon={PlusIcon} />
				Add Subject
			</Button>
		</li>
	</menu>
</header>

<main>
	<SubjectsSection
		subjects={data}
		onEditSubject={(subjectId, name) => (activeDialog = { type: 'editSubject', subjectId, name })}
		onDeleteSubject={deleteSubject}
		onAddGrade={(subjectId) => (activeDialog = { type: 'addGrade', subjectId })}
		onDeleteGrade={deleteGrade}
		onAddExam={(subjectId, grade, paper) =>
			(activeDialog = { type: 'addExam', subjectId, grade, paper })}
		onEditExam={(examRow) => (activeDialog = { type: 'editExam', exam: examRow })}
	/>
</main>

<AddSubjectDialog
	open={activeDialog?.type === 'addSubject'}
	onClose={closeDialog}
	onSaved={refreshSubjects}
/>

<EditSubjectDialog
	open={activeDialog?.type === 'editSubject'}
	subjectId={activeDialog?.type === 'editSubject' ? activeDialog.subjectId : 0}
	name={activeDialog?.type === 'editSubject' ? activeDialog.name : ''}
	onClose={closeDialog}
	onSaved={refreshSubjects}
/>

<AddGradeDialog
	open={activeDialog?.type === 'addGrade'}
	subjectId={activeDialog?.type === 'addGrade' ? activeDialog.subjectId : 0}
	onClose={closeDialog}
	onSaved={refreshSubjects}
/>

<AddExamDialog
	open={activeDialog?.type === 'addExam'}
	subjectId={activeDialog?.type === 'addExam' ? activeDialog.subjectId : 0}
	grade={activeDialog?.type === 'addExam' ? activeDialog.grade : 8}
	paper={activeDialog?.type === 'addExam' ? activeDialog.paper : 1}
	{allTimeslots}
	onClose={closeDialog}
	onSaved={refreshSubjects}
/>

<EditExamDialog
	open={activeDialog?.type === 'editExam'}
	examId={activeDialog?.type === 'editExam' ? activeDialog.exam.id : 0}
	durationHours={activeDialog?.type === 'editExam' ? activeDialog.exam.durationHours : 2}
	slotsRequired={activeDialog?.type === 'editExam' ? activeDialog.exam.slotsRequired : 1}
	priority={activeDialog?.type === 'editExam' ? activeDialog.exam.priority : 0}
	{allTimeslots}
	onClose={closeDialog}
	onSaved={refreshSubjects}
/>
