<script lang="ts">
	import { db } from '$lib/db';
	import { exam, examConstraint, subject, type ExamConstraintType } from '$lib/db/schema';
	import { and, asc, eq } from 'drizzle-orm';
	import { Button, Card } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import { pipe, string, nonEmpty, picklist } from 'valibot';
	import { formatExamLabel } from '$lib/examDisplay';
	import EnhancedSelect from '$lib/EnhancedSelect.svelte';
	import { alias } from 'drizzle-orm/sqlite-core';

	const CONSTRAINT_TYPE_NAMES: Record<ExamConstraintType, string> = {
		same_time: 'Same Time',
		same_day: 'Same Day',
		same_week: 'Same Week',
		different_time: 'Different Time',
		different_day: 'Different Day',
		different_week: 'Different Week',
		before: 'Before'
	};

	const CONSTRAINT_TYPE_OPTIONS: { value: ExamConstraintType; text: string }[] = Object.entries(
		CONSTRAINT_TYPE_NAMES
	).map(([value, text]) => ({ value: value as ExamConstraintType, text }));

	const CONSTRAINT_TYPE_DESCRIPTIONS: Record<ExamConstraintType, string> = {
		same_time: 'These two exams must happen in the exact same timeslot.',
		same_day: 'These two exams must happen on the same date, consecutively.',
		same_week: 'These two exams must be scheduled in the same calendar week.',
		different_time: 'These two exams must happen at different times.',
		different_day: 'These two exams must happen on different dates.',
		different_week: 'These two exams must be scheduled in different calendar weeks.',
		before: 'The first exam must be scheduled before the second exam.',
	};

	const firstExam = alias(exam, 'firstExam');
	const secondExam = alias(exam, 'secondExam');
	const firstSubject = alias(subject, 'firstSubject');
	const secondSubject = alias(subject, 'secondSubject');

	async function getAllConstraints() {
		// Turns out, you can't really orderBy a with: {} in drizzle-orm. It only works on array/many joins
		const rows = await db
			.select()
			.from(examConstraint)
			.innerJoin(firstExam, eq(examConstraint.exam1Id, firstExam.id))
			.innerJoin(firstSubject, eq(firstExam.subjectId, firstSubject.id))
			.innerJoin(secondExam, eq(examConstraint.exam2Id, secondExam.id))
			.innerJoin(secondSubject, eq(secondExam.subjectId, secondSubject.id))
			.orderBy(asc(firstExam.grade), asc(firstSubject.name), asc(firstExam.paper));

		// Reshape flat rows into nested structure
		return rows.map((row) => ({
			...row.exam_constraint,
			firstExam: { ...row.firstExam, subject: row.firstSubject },
			secondExam: { ...row.secondExam, subject: row.secondSubject }
		}));
	}

	/** AI-generated (GPT-5.3-codex). */
	function normalizePair(examAId: number, examBId: number): [number, number] {
		return examAId < examBId ? [examAId, examBId] : [examBId, examAId];
	}

	/** AI-generated (GPT-5.3-codex). */
	function toConstraintKey(exam1Id: number, exam2Id: number): string {
		const [left, right] = normalizePair(exam1Id, exam2Id);
		return `${left}-${right}`;
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getSubjectOptions() {
		return (await db.query.subject.findMany({ orderBy: { name: 'asc' } })).map((subject) => ({
			value: subject.id.toString(),
			text: subject.name
		}));
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getGradeOptionsForSelectedSubject(subjectId: string) {
		const parsedSubjectId = Number(subjectId);
		if (!Number.isFinite(parsedSubjectId)) return [];

		const grades = await db.query.subjectGrade.findMany({
			where: { subjectId: parsedSubjectId },
			orderBy: { grade: 'asc' }
		});

		return grades.map((grade) => ({
			value: grade.grade.toString(),
			text: `Grade ${grade.grade}`
		}));
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getPaperOptionsForSelectedSubjectAndGrade(subjectId: string, grade: string) {
		const parsedSubjectId = Number(subjectId);
		const parsedGrade = Number(grade);
		if (!Number.isFinite(parsedSubjectId) || !parsedGrade) return [];

		const papers = await db.query.exam.findMany({
			where: { subjectId: parsedSubjectId, grade: parsedGrade },
			with: { subject: true },
			orderBy: { paper: 'asc' }
		});

		return papers.map((paper) => ({
			value: paper.id.toString(),
			text: formatExamLabel(paper)
		}));
	}

	async function refreshConstraints() {
		constraints = await getAllConstraints();
	}

	let constraints = $state(await getAllConstraints());

	const groupedConstraints = $derived(
		['same_time', 'same_day', 'same_week', 'different_time', 'different_day', 'different_week']
			.map((type) => ({
				type: type as ExamConstraintType,
				constraints: constraints.filter((c) => c.constraintType === type)
			}))
			.filter((group) => group.constraints.length > 0)
	);
	const subjectOptions = await getSubjectOptions();

	const constraintKeys = $derived(
		new Set(constraints.map((c) => toConstraintKey(c.exam1Id, c.exam2Id)))
	);

	const addConstraintForm = createForm(() => ({
		defaultValues: {
			constraintType: 'same_time' as ExamConstraintType,
			grade1: '',
			subject1Id: '',
			exam1Id: '',
			grade2: '',
			subject2Id: '',
			exam2Id: ''
		},
		validators: {
			onChange: ({ value }) => {
				if (!value.exam1Id || !value.exam2Id) return undefined;
				if (value.exam1Id === value.exam2Id) return 'Pick two different exams.';

				if (constraintKeys.has(toConstraintKey(Number(value.exam1Id), Number(value.exam2Id)))) {
					return 'This constraint already exists.';
				}

				return undefined;
			}
		},
		onSubmit: async ({ value, formApi }) => {
			const exam1Id = Number(value.exam1Id);
			const exam2Id = Number(value.exam2Id);
			const constraintType = value.constraintType as ExamConstraintType;

			if (!Number.isFinite(exam1Id) || !Number.isFinite(exam2Id)) return;
			if (exam1Id === exam2Id) return;
			if (!constraintType) return;

			const [normalizedExam1Id, normalizedExam2Id] = normalizePair(exam1Id, exam2Id);

			await db
				.insert(examConstraint)
				.values({
					exam1Id: normalizedExam1Id,
					exam2Id: normalizedExam2Id,
					constraintType
				})
				.onConflictDoNothing();

			await refreshConstraints();
			formApi.reset();
		}
	}));

	/** AI-generated (GPT-5.3-codex). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		addConstraintForm.handleSubmit();
	}

	async function removeConstraint(exam1Id: number, exam2Id: number) {
		if (!confirm('Are you sure you want to delete this constraint? This action cannot be undone.'))
			return;
		await db
			.delete(examConstraint)
			.where(and(eq(examConstraint.exam1Id, exam1Id), eq(examConstraint.exam2Id, exam2Id)));
		await refreshConstraints();
	}

	const counts = $derived({
		same_time: constraints.filter((c) => c.constraintType === 'same_time').length,
		same_day: constraints.filter((c) => c.constraintType === 'same_day').length,
		same_week: constraints.filter((c) => c.constraintType === 'same_week').length,
		different_time: constraints.filter((c) => c.constraintType === 'different_time').length,
		different_day: constraints.filter((c) => c.constraintType === 'different_day').length,
		different_week: constraints.filter((c) => c.constraintType === 'different_week').length
	});
</script>

<header class="constraints-header">
	<div>
		<h1>Constraints</h1>
		<h2>Setup additional constraints for the timetable</h2>
	</div>
	<ul aria-label="Constraint counts">
		<li>Same-time: {counts.same_time}</li>
		<li>Same-day: {counts.same_day}</li>
		<li>Same-week: {counts.same_week}</li>
		<li>Different-time: {counts.different_time}</li>
		<li>Different-day: {counts.different_day}</li>
		<li>Different-week: {counts.different_week}</li>
	</ul>
</header>

<main>
	<Card variant="filled">
		<section class="add-constraint-card">
			<header>
				<h2>Add Constraints</h2>
				<p>Define relationships between pairs of exams.</p>
			</header>

			<form id="add-constraint-form" onsubmit={handleFormSubmit}>
				<fieldset>
					<legend>Constraint Type</legend>
					<addConstraintForm.Field
						name="constraintType"
						validators={{
							onChange: pipe(
								picklist(
									[
										'same_day',
										'different_day',
										'same_week',
										'different_week',
										'same_time',
										'different_time'
									] satisfies ExamConstraintType[],
									'Pick a constraint type.'
								)
							)
						}}
					>
						{#snippet children(field)}
							<EnhancedSelect
								{field}
								variant="outlined"
								label="Type"
								options={CONSTRAINT_TYPE_OPTIONS}
							/>
						{/snippet}
					</addConstraintForm.Field>

					<addConstraintForm.Subscribe selector={(state) => state.values.constraintType}>
						{#snippet children(constraintType)}
							<p class="constraint-description">
								{CONSTRAINT_TYPE_DESCRIPTIONS[constraintType]}
							</p>
						{/snippet}
					</addConstraintForm.Subscribe>
				</fieldset>

				<div class="exams">
					<fieldset>
						<legend>First Exam</legend>

						<addConstraintForm.Field
							name="subject1Id"
							validators={{
								onChange: pipe(string(), nonEmpty('Pick subject for exam 1.'))
							}}
						>
							{#snippet children(field)}
								<EnhancedSelect
									{field}
									variant="outlined"
									label="Subject A"
									options={subjectOptions}
									onValueChange={() => {
										addConstraintForm.setFieldValue('grade1', '');
										addConstraintForm.setFieldValue('exam1Id', '');
									}}
								/>
							{/snippet}
						</addConstraintForm.Field>

						<addConstraintForm.Subscribe selector={(state) => state.values.subject1Id}>
							{#snippet children(subject1Id)}
								<addConstraintForm.Field
									name="grade1"
									validators={{
										onChange: pipe(string(), nonEmpty('Pick grade for exam 1.'))
									}}
								>
									{#snippet children(field)}
										<EnhancedSelect
											{field}
											variant="outlined"
											label="Grade 1"
											options={await getGradeOptionsForSelectedSubject(subject1Id)}
											onValueChange={() => {
												addConstraintForm.setFieldValue('exam1Id', '');
											}}
											disabled={!subject1Id}
										/>
									{/snippet}
								</addConstraintForm.Field>
							{/snippet}
						</addConstraintForm.Subscribe>

						<addConstraintForm.Subscribe
							selector={(state) => ({
								grade1: state.values.grade1,
								subject1Id: state.values.subject1Id
							})}
						>
							{#snippet children(state)}
								<addConstraintForm.Field
									name="exam1Id"
									validators={{
										onChange: pipe(string(), nonEmpty('Pick paper for exam 1.'))
									}}
								>
									{#snippet children(field)}
										<EnhancedSelect
											{field}
											variant="outlined"
											label="Paper 1"
											options={await getPaperOptionsForSelectedSubjectAndGrade(
												state.subject1Id,
												state.grade1
											)}
											disabled={!state.grade1 || !state.subject1Id}
										/>
									{/snippet}
								</addConstraintForm.Field>
							{/snippet}
						</addConstraintForm.Subscribe>
					</fieldset>

					<fieldset>
						<legend>Second Exam</legend>

						<addConstraintForm.Field
							name="subject2Id"
							validators={{
								onChange: pipe(string(), nonEmpty('Pick subject for exam 2.'))
							}}
						>
							{#snippet children(field)}
								<EnhancedSelect
									{field}
									variant="outlined"
									label="Subject 2"
									options={subjectOptions}
									onValueChange={() => {
										addConstraintForm.setFieldValue('grade2', '');
										addConstraintForm.setFieldValue('exam2Id', '');
									}}
								/>
							{/snippet}
						</addConstraintForm.Field>

						<addConstraintForm.Subscribe selector={(state) => state.values.subject2Id}>
							{#snippet children(subject2Id)}
								<addConstraintForm.Field
									name="grade2"
									validators={{
										onChange: pipe(string(), nonEmpty('Pick grade for exam 2.'))
									}}
								>
									{#snippet children(field)}
										<EnhancedSelect
											{field}
											variant="outlined"
											label="Grade 2"
											options={await getGradeOptionsForSelectedSubject(subject2Id)}
											onValueChange={() => {
												addConstraintForm.setFieldValue('exam2Id', '');
											}}
											disabled={!subject2Id}
										/>
									{/snippet}
								</addConstraintForm.Field>
							{/snippet}
						</addConstraintForm.Subscribe>

						<addConstraintForm.Subscribe
							selector={(state) => ({
								grade2: state.values.grade2,
								subject2Id: state.values.subject2Id
							})}
						>
							{#snippet children(state)}
								<addConstraintForm.Field
									name="exam2Id"
									validators={{
										onChange: pipe(string(), nonEmpty('Pick paper for exam 2.'))
									}}
								>
									{#snippet children(field)}
										<EnhancedSelect
											{field}
											variant="outlined"
											label="Paper 2"
											options={await getPaperOptionsForSelectedSubjectAndGrade(
												state.subject2Id,
												state.grade2
											)}
											disabled={!state.grade2 || !state.subject2Id}
										/>
									{/snippet}
								</addConstraintForm.Field>
							{/snippet}
						</addConstraintForm.Subscribe>
					</fieldset>
				</div>
			</form>

			<footer>
				<addConstraintForm.Subscribe
					selector={(state) => ({
						canSubmit: state.canSubmit,
						isSubmitting: state.isSubmitting,
						isPristine: state.isPristine
					})}
				>
					{#snippet children(state)}
						<Button
							form="add-constraint-form"
							type="submit"
							variant="filled"
							disabled={!state.canSubmit || state.isSubmitting || state.isPristine}
						>
							Add Constraint
						</Button>
					{/snippet}
				</addConstraintForm.Subscribe>
			</footer>
		</section>
	</Card>

	<Card variant="filled">
		<section class="constraints-card">
			<header>
				<h2>Constraints</h2>
			</header>

			{#if constraints.length === 0}
				<p>No constraints yet.</p>
			{:else}
				{#each groupedConstraints as group (group.type)}
					<table>
						<thead>
							<tr>
								<th>{CONSTRAINT_TYPE_NAMES[group.type]} Exams</th>
								<th>Action</th>
							</tr>
						</thead>
						<tbody>
							{#each group.constraints as constraint (toConstraintKey(constraint.exam1Id, constraint.exam2Id))}
								<tr>
									<td
										>{formatExamLabel(constraint.firstExam)} — {formatExamLabel(
											constraint.secondExam
										)}</td
									>
									<td>
										<Button
											variant="text"
											color="error"
											onclick={() => removeConstraint(constraint.exam1Id, constraint.exam2Id)}
										>
											Delete
										</Button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				{/each}
			{/if}
		</section>
	</Card>
</main>

<style>
	section {
		border-radius: var(--m3-shape-medium);
	}

	table {
		margin-top: 1rem;
		width: 100%;
		border-radius: var(--m3-shape-medium);
		border-collapse: collapse;
		background-color: var(--m3c-surface-container-low, var(--m3c-surface));
		outline: 1px solid var(--m3c-outline-variant);

		& thead tr {
			background-color: var(--m3c-surface-container-highest);
			border-bottom: 1px solid var(--m3c-outline-variant);
		}

		& th {
			@apply --m3-label-large;
			color: var(--m3c-on-surface-variant);
			text-align: left;
			padding: 0.875rem 1rem;
			white-space: nowrap;
			user-select: none;
		}

		& tbody tr {
			border-bottom: 1px solid var(--m3c-outline-variant);
			transition: background-color var(--m3-easing-fast, 150ms ease);

			&:last-child {
				border-bottom: none;
			}

			&:hover {
				background-color: color-mix(in oklch, var(--m3c-on-surface) 8%, transparent);
			}
		}

		& td {
			@apply --m3-body-medium;
			color: var(--m3c-on-surface);
			padding: 0.75rem 1rem;
			vertical-align: middle;
		}
	}

	header.constraints-header {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 1rem;
		align-items: end;

		& > ul {
			display: flex;
			gap: 0.5rem;
			flex-wrap: wrap;
			list-style: none;
			padding: 0;
		}

		& li {
			@apply --m3-label-large;
			display: inline-flex;
			align-items: center;
			border-radius: var(--m3-shape-full);
			padding: 0.375rem 0.75rem;
			background: var(--m3c-secondary-container);
			color: var(--m3c-on-secondary-container);
		}
	}

	main {
		display: grid;
		gap: 1rem;
	}

	.add-constraint-card {
		display: grid;
		gap: 0.75rem;

		& > header {
			display: grid;
			gap: 0.25rem;
			align-items: start;
		}

		& > header > h2 {
			@apply --m3-title-large;
			color: var(--m3c-on-surface);
		}

		& > header > p {
			@apply --m3-body-medium;
			color: var(--m3c-on-surface-variant);
		}

		& > form {
			display: flex;
			flex-direction: column;

			gap: 1rem;

			& .exams {
				display: flex;
				gap: 1rem;

				@media (max-width: 600px) {
					flex-direction: column;
				}
			}
		}

		& fieldset {
			flex-grow: 1;
			display: grid;
			gap: 0.75rem;
			padding: 1rem;
			border: 1px solid var(--m3c-outline-variant);
			border-radius: 1rem;
			background: var(--m3c-surface-container-highest);
		}

		& legend {
			@apply --m3-title-small;
			color: var(--m3c-on-surface-variant);
			padding-inline: 0.25rem;
		}

		& > footer {
			margin-top: 0.75rem;
			display: flex;
			gap: 0.75rem;
			align-items: center;
			flex-wrap: wrap;
		}
	}

	@media (width >= 60rem) {
		main {
			gap: 1.25rem;
		}

		.add-constraint-card {
			& > form {
				gap: 1.25rem;
			}

			& > footer {
				margin-top: 1rem;
			}
		}
	}
</style>
