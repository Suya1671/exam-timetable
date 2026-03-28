<script lang="ts">
	import { db } from '$lib/db';
	import { sameTimeExam } from '$lib/db/schema';
	import { and, eq } from 'drizzle-orm';
	import { Button, Card } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import { pipe, string, nonEmpty } from 'valibot';
	import { formatExamLabel } from '$lib/examDisplay';
	import EnhancedSelect from '$lib/EnhancedSelect.svelte';

	let { onCountChange }: { onCountChange?: (count: number) => void } = $props();

	/** AI-generated (GPT-5.3-codex). */
	async function getSameTimeExams() {
		return await db.query.sameTimeExam.findMany({
			with: {
				firstExam: {
					with: {
						subject: true
					}
				},
				secondExam: {
					with: {
						subject: true
					}
				}
			},
			orderBy: {
				exam1Id: 'asc',
				exam2Id: 'asc'
			}
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	function normalizePair(examAId: number, examBId: number): [number, number] {
		return examAId < examBId ? [examAId, examBId] : [examBId, examAId];
	}

	/** AI-generated (GPT-5.3-codex). */
	function toPairKey(examAId: number, examBId: number): string {
		const [left, right] = normalizePair(examAId, examBId);
		return `${left}-${right}`;
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getSubjectOptions() {
		return (await db.query.subject.findMany({})).map((subject) => ({
			value: subject.id.toString(),
			text: subject.name
		}));
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getGradeOptionsForSelectedSubject(subjectId: string) {
		const parsedSubjectId = Number(subjectId);
		if (!Number.isFinite(parsedSubjectId)) return [];

		const grades = await db.query.subjectGrade.findMany({
			where: {
				subjectId: parsedSubjectId
			}
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
			where: {
				subjectId: parsedSubjectId,
				grade: parsedGrade
			},
			with: {
				subject: true
			}
		});

		return papers.map((paper) => ({
			value: paper.id.toString(),
			text: formatExamLabel(paper)
		}));
	}

	/** AI-generated (GPT-5.3-codex). */
	async function refreshSameTimeExams() {
		sameTimeExams = await getSameTimeExams();
	}

	let sameTimeExams = $state(await getSameTimeExams());
	const subjectOptions = await getSubjectOptions();

	const sameTimePairKeys = $derived(
		new Set(sameTimeExams.map((pair) => toPairKey(pair.exam1Id, pair.exam2Id)))
	);

	const addSameTimeForm = createForm(() => ({
		defaultValues: {
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

				const pairKey = toPairKey(Number(value.exam1Id), Number(value.exam2Id));
				if (sameTimePairKeys.has(pairKey)) return 'This pair already exists.';

				return undefined;
			}
		},
		onSubmit: async ({ value, formApi }) => {
			const exam1Id = Number(value.exam1Id);
			const exam2Id = Number(value.exam2Id);

			if (!Number.isFinite(exam1Id) || !Number.isFinite(exam2Id)) return;
			if (exam1Id === exam2Id) return;

			const [firstSlotExamId, secondSlotExamId] = normalizePair(exam1Id, exam2Id);

			await db
				.insert(sameTimeExam)
				.values({ exam1Id: firstSlotExamId, exam2Id: secondSlotExamId })
				.onConflictDoNothing();

			await refreshSameTimeExams();
			formApi.reset();
		}
	}));

	/** AI-generated (GPT-5.3-codex). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		addSameTimeForm.handleSubmit();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function removeSameTimeConstraint(exam1Id: number, exam2Id: number) {
		await db
			.delete(sameTimeExam)
			.where(and(eq(sameTimeExam.exam1Id, exam1Id), eq(sameTimeExam.exam2Id, exam2Id)));
		await refreshSameTimeExams();
	}

	$effect(() => {
		onCountChange?.(sameTimeExams.length);
	});
</script>

<Card variant="filled">
	<section data-constraint-card>
		<header>
			<h2>Same Time Exams</h2>
			<p>These two exams must happen in the exact same timeslot.</p>
		</header>

		<form id="add-same-time-constraint-form" onsubmit={handleFormSubmit}>
			<fieldset>
				<legend>First Exam</legend>

				<addSameTimeForm.Field
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
								addSameTimeForm.setFieldValue('grade1', '');
								addSameTimeForm.setFieldValue('exam1Id', '');
							}}
						/>
					{/snippet}
				</addSameTimeForm.Field>

				<addSameTimeForm.Subscribe selector={(state) => state.values.subject1Id}>
					{#snippet children(subject1Id)}
						<addSameTimeForm.Field
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
										addSameTimeForm.setFieldValue('exam1Id', '');
									}}
									disabled={!subject1Id}
								/>
							{/snippet}
						</addSameTimeForm.Field>
					{/snippet}
				</addSameTimeForm.Subscribe>

				<addSameTimeForm.Subscribe
					selector={(state) => ({
						grade1: state.values.grade1,
						subject1Id: state.values.subject1Id
					})}
				>
					{#snippet children(state)}
						<addSameTimeForm.Field
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
						</addSameTimeForm.Field>
					{/snippet}
				</addSameTimeForm.Subscribe>
			</fieldset>

			<fieldset>
				<legend>Second Exam</legend>

				<addSameTimeForm.Field
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
								addSameTimeForm.setFieldValue('grade2', '');
								addSameTimeForm.setFieldValue('exam2Id', '');
							}}
						/>
					{/snippet}
				</addSameTimeForm.Field>

				<addSameTimeForm.Subscribe selector={(state) => state.values.subject2Id}>
					{#snippet children(subject2Id)}
						<addSameTimeForm.Field
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
										addSameTimeForm.setFieldValue('exam2Id', '');
									}}
									disabled={!subject2Id}
								/>
							{/snippet}
						</addSameTimeForm.Field>
					{/snippet}
				</addSameTimeForm.Subscribe>

				<addSameTimeForm.Subscribe
					selector={(state) => ({
						grade2: state.values.grade2,
						subject2Id: state.values.subject2Id
					})}
				>
					{#snippet children(state)}
						<addSameTimeForm.Field
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
						</addSameTimeForm.Field>
					{/snippet}
				</addSameTimeForm.Subscribe>
			</fieldset>
		</form>

		<footer>
			<addSameTimeForm.Subscribe
				selector={(state) => ({
					canSubmit: state.canSubmit,
					isSubmitting: state.isSubmitting,
					isPristine: state.isPristine
				})}
			>
				{#snippet children(state)}
					<Button
						form="add-same-time-constraint-form"
						type="submit"
						variant="filled"
						disabled={!state.canSubmit || state.isSubmitting || state.isPristine}
					>
						Add
					</Button>
				{/snippet}
			</addSameTimeForm.Subscribe>
		</footer>

		{#if sameTimeExams.length === 0}
			<p>No same-time constraints yet.</p>
		{:else}
			<ul>
				{#each sameTimeExams as pair (toPairKey(pair.exam1Id, pair.exam2Id))}
					<li>
						<span>{formatExamLabel(pair.firstExam)} — {formatExamLabel(pair.secondExam)}</span>
						<Button
							variant="text"
							color="error"
							onclick={() => removeSameTimeConstraint(pair.exam1Id, pair.exam2Id)}
						>
							Delete
						</Button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</Card>
