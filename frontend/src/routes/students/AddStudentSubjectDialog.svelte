<script lang="ts">
	import { db } from '$lib/db';
	import { enrolledStudent, student, subject, subjectGrade } from '$lib/db/schema';
	import { and, eq, isNull } from 'drizzle-orm';
	import { Button, Dialog } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import { nonEmpty, pipe, string } from 'valibot';
	import EnhancedSelect from '$lib/EnhancedSelect.svelte';

	let {
		open = false,
		studentId = 0,
		onClose,
		onSaved
	}: {
		open?: boolean;
		studentId?: number;
		onClose?: () => void;
		onSaved?: () => void | Promise<void>;
	} = $props();

	let previousOpen = $state(false);

	const addSubjectForm = createForm(() => ({
		defaultValues: {
			studentId: '0',
			subjectId: '0'
		},
		onSubmit: async ({ value, formApi }) => {
			const resolvedStudentId = Number(value.studentId);
			const subjectId = Number(value.subjectId);
			if (!Number.isFinite(resolvedStudentId) || !Number.isFinite(subjectId)) return;

			await db
				.insert(enrolledStudent)
				.values({ studentId: resolvedStudentId, subjectId })
				.onConflictDoNothing();
			await onSaved?.();
			onClose?.();
			formApi.reset();
		}
	}));

	/** AI-generated (GPT-5.2-codex). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		addSubjectForm.handleSubmit();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getAvailableSubjectsForStudent(studentId: number) {
		return db
			.select({ id: subject.id, name: subject.name })
			.from(subject)
			.innerJoin(
				subjectGrade,
				and(eq(subjectGrade.subjectId, subject.id), eq(subjectGrade.grade, student.grade))
			)
			.innerJoin(student, eq(student.id, studentId))
			.leftJoin(
				enrolledStudent,
				and(eq(enrolledStudent.subjectId, subject.id), eq(enrolledStudent.studentId, student.id))
			)
			.where(isNull(enrolledStudent.studentId));
	}

	$effect(() => {
		if (open && !previousOpen) {
			addSubjectForm.setFieldValue('studentId', studentId.toString());
			addSubjectForm.setFieldValue('subjectId', '0');
		}

		previousOpen = open;
	});
</script>

<Dialog
	headline="Add Subject"
	bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
	{@const availableSubjects =
		open && Number.isFinite(studentId) && studentId > 0
			? await getAvailableSubjectsForStudent(studentId)
			: []}
	{@const availableSubjectOptions = availableSubjects.map((availableSubject) => ({
		value: availableSubject.id.toString(),
		text: availableSubject.name
	}))}

	<form id="add-student-subject-form" onsubmit={handleFormSubmit}>
		<addSubjectForm.Field
			name="subjectId"
			validators={{ onChange: pipe(string(), nonEmpty('Pick a subject.')) }}
		>
			{#snippet children(field)}
				<EnhancedSelect
					{field}
					label="Subject"
					required
					variant="outlined"
					options={availableSubjectOptions}
				/>
			{/snippet}
		</addSubjectForm.Field>
	</form>

	{#snippet buttons()}
		<Button variant="outlined" onclick={onClose}>Cancel</Button>
		<addSubjectForm.Subscribe
			selector={(state) => ({
				canSubmit: state.canSubmit,
				isSubmitting: state.isSubmitting
			})}
		>
			{#snippet children(state)}
				<Button
					form="add-student-subject-form"
					type="submit"
					disabled={!state.canSubmit || state.isSubmitting}>Add</Button
				>
			{/snippet}
		</addSubjectForm.Subscribe>
	{/snippet}
</Dialog>

<style>
	form {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
