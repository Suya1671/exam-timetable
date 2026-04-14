<script lang="ts">
	import { db } from '$lib/db';
	import { exam, examTimeslotRestriction, timeslot } from '$lib/db/schema';
	import { eq } from 'drizzle-orm';
	import { SvelteSet } from 'svelte/reactivity';
	import { Button, Dialog } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import EnhancedTextInput from '$lib/EnhancedTextInput.svelte';
	import TimeslotPicker from '$lib/TimeslotPicker.svelte';
	import { durationHoursValidator, priorityValidator, slotsRequiredValidator } from './forms';
	import { string } from 'valibot';

	type RestrictionMode = 'allow' | 'deny';

	let {
		open = false,
		subjectId = 0,
		grade = 8,
		paper = 1,
		allTimeslots,
		onClose,
		onSaved
	}: {
		open?: boolean;
		subjectId?: number;
		grade?: number;
		paper?: number;
		allTimeslots: Array<typeof timeslot.$inferSelect>;
		onClose?: () => void;
		onSaved?: () => void | Promise<void>;
	} = $props();

	let restrictionIds = new SvelteSet<number>();
	let restrictionMode = $state<RestrictionMode>('deny');
	let previousOpen = $state(false);

	const addExamForm = createForm(() => ({
		defaultValues: {
			subjectId: 0,
			paper: 1,
			grade: 8,
			slotsRequired: 1,
			durationHours: 2,
			name: '',
			priority: 0
		} satisfies typeof exam.$inferInsert,
		onSubmit: async ({ value, formApi }) => {
			value.name = value.name?.trim();
			const newValue = {
				...value,
				name: value.name == '' ? null : value.name
			};

			const [createdExam] = await db.insert(exam).values(newValue).returning({ id: exam.id });
			if (createdExam) {
				await replaceExamTimeslotRestrictions(createdExam.id, restrictionIds, restrictionMode);
			}

			await onSaved?.();
			onClose?.();
			restrictionIds.clear();
			restrictionMode = 'deny';
			formApi.reset();
		}
	}));

	/** AI-generated (GPT-5.3-codex). */
	async function getSubjectName(subjectId: number) {
		return db.query.subject.findFirst({
			where: { id: subjectId },
			columns: { name: true }
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	async function replaceExamTimeslotRestrictions(
		examId: number,
		selectedIds: Set<number>,
		mode: RestrictionMode
	) {
		const normalizedIds = new Set(selectedIds);
		const persistedMode = normalizedIds.size > 0 ? mode : null;

		await db.transaction(async (tx) => {
			await tx
				.update(exam)
				.set({ timeslotRestrictionMode: persistedMode })
				.where(eq(exam.id, examId));
			await tx.delete(examTimeslotRestriction).where(eq(examTimeslotRestriction.examId, examId));
			if (normalizedIds.size > 0) {
				await tx
					.insert(examTimeslotRestriction)
					.values([...normalizedIds].map((timeslotId) => ({ examId, timeslotId })));
			}
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		addExamForm.handleSubmit();
	}

	$effect(() => {
		if (!(open && !previousOpen)) {
			previousOpen = open;
			return;
		}

		addExamForm.setFieldValue('subjectId', subjectId);
		addExamForm.setFieldValue('grade', grade);
		addExamForm.setFieldValue('paper', paper);
		addExamForm.setFieldValue('durationHours', 2);
		addExamForm.setFieldValue('slotsRequired', 1);
		addExamForm.setFieldValue('priority', 0);
		addExamForm.setFieldValue('name', '');

		restrictionMode = 'deny';
		restrictionIds.clear();
		previousOpen = open;
	});
</script>

<addExamForm.Subscribe
	selector={(state) => ({
		subjectId: state.values.subjectId,
		grade: state.values.grade,
		paper: state.values.paper
	})}
>
	{#snippet children(state)}
		{@const subjectData = await getSubjectName(state.subjectId)}
		<Dialog
			headline={`Creating ${subjectData?.name} (Grade ${state.grade}) Paper ${state.paper}`}
			bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
		>
			<form id="add-exam-form" onsubmit={handleFormSubmit}>
				<addExamForm.Field name="name" validators={{ onChange: string() }}>
					{#snippet children(field)}
						<EnhancedTextInput
							{field}
							label="Exam Name (Optional)"
							placeholder="Enter exam name"
							helperText="Optional exam name. Exams are by default named after the subject and paper number."
						/>
					{/snippet}
				</addExamForm.Field>

				<addExamForm.Field name="durationHours" validators={{ onChange: durationHoursValidator }}>
					{#snippet children(field)}
						<EnhancedTextInput
							{field}
							label="Duration (hours)"
							type="number"
							min="0.5"
							step="0.25"
							required
							helperText="How long the exam lasts for (in hours)"
						/>
					{/snippet}
				</addExamForm.Field>

				<addExamForm.Field name="slotsRequired" validators={{ onChange: slotsRequiredValidator }}>
					{#snippet children(field)}
						<EnhancedTextInput
							{field}
							label="Slots Required"
							type="number"
							min="1"
							required
							helperText="How many timeslots this exam runs over. For most exams, this will be 1"
						/>
					{/snippet}
				</addExamForm.Field>

				<addExamForm.Field name="priority" validators={{ onChange: priorityValidator }}>
					{#snippet children(field)}
						<EnhancedTextInput
							{field}
							label="Priority"
							type="number"
							min="0"
							step="1"
							required
							helperText="Higher priority exams will be scheduled earlier when possible"
						/>
					{/snippet}
				</addExamForm.Field>

				<fieldset>
					<legend>Timeslot restrictions</legend>
					<p>
						Add exact slots this exam is allowed or denied to be scheduled in. If no timeslots are
						allowed/denied, the exam can be scheduled in any slot. If there are only denied slots,
						the exam can be scheduled in any slot except the denied ones.
					</p>
					<TimeslotPicker {allTimeslots} selectedIds={restrictionIds} mode={restrictionMode} />
				</fieldset>
			</form>

			{#snippet buttons()}
				<Button variant="outlined" onclick={onClose}>Cancel</Button>
				<Button form="add-exam-form" type="submit">Add</Button>
			{/snippet}
		</Dialog>
	{/snippet}
</addExamForm.Subscribe>

<style>
	form {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	fieldset {
		border: 1px solid var(--m3c-outline-variant);
		border-radius: var(--m3-shape-medium);
		background-color: var(--m3c-surface);
		padding: 1rem 1.5rem 1.5rem;
		margin: 0;

		& legend {
			padding-inline: 0.25rem;
			@apply --m3-label-large;
		}

		& p {
			@apply --m3-body-medium;
			color: var(--m3c-on-surface-variant);
			margin-bottom: 1rem;
		}
	}
</style>
