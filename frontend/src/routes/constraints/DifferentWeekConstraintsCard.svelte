<script lang="ts">
	import { db } from '$lib/db';
	import { differentWeekExams } from '$lib/db/schema';
	import { and, eq, or } from 'drizzle-orm';
	import { Button, Card } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import { pipe, string, nonEmpty } from 'valibot';
	import { formatExamLabel } from '$lib/examDisplay';
	import EnhancedSelect from '$lib/EnhancedSelect.svelte';

	let { onCountChange }: { onCountChange?: (count: number) => void } = $props();

	/** AI-generated (GPT-5.3-codex). */
	async function getDifferentWeekExams() {
		return await db.query.differentWeekExams.findMany({
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
	async function refreshDifferentWeekExams() {
		differentWeekExamPairs = await getDifferentWeekExams();
	}

	let differentWeekExamPairs = $state(await getDifferentWeekExams());
	const subjectOptions = await getSubjectOptions();

	const differentWeekPairKeys = $derived(
		new Set(differentWeekExamPairs.map((pair) => toPairKey(pair.exam1Id, pair.exam2Id)))
	);

	const addDifferentWeekForm = createForm(() => ({
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
				if (differentWeekPairKeys.has(pairKey)) return 'This pair already exists.';

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
				.insert(differentWeekExams)
				.values({ exam1Id: firstSlotExamId, exam2Id: secondSlotExamId })
				.onConflictDoNothing();

			await refreshDifferentWeekExams();
			formApi.reset();
		}
	}));

	/** AI-generated (GPT-5.3-codex). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		addDifferentWeekForm.handleSubmit();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function removeDifferentWeekConstraint(exam1Id: number, exam2Id: number) {
		await db
			.delete(differentWeekExams)
			.where(
				or(
					and(eq(differentWeekExams.exam1Id, exam1Id), eq(differentWeekExams.exam2Id, exam2Id)),
					and(eq(differentWeekExams.exam1Id, exam2Id), eq(differentWeekExams.exam2Id, exam1Id))
				)
			);
		await refreshDifferentWeekExams();
	}

	$effect(() => {
		onCountChange?.(differentWeekExamPairs.length);
	});
</script>

<Card variant="filled">
	<section data-constraint-card>
		<header>
			<h2>Different Week Exams</h2>
			<p>These two exams must be scheduled in different calendar weeks.</p>
		</header>

		<form id="add-different-week-constraint-form" onsubmit={handleFormSubmit}>
			<fieldset>
				<legend>First Exam</legend>

				<addDifferentWeekForm.Field
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
								addDifferentWeekForm.setFieldValue('grade1', '');
								addDifferentWeekForm.setFieldValue('exam1Id', '');
							}}
						/>
					{/snippet}
				</addDifferentWeekForm.Field>

				<addDifferentWeekForm.Subscribe selector={(state) => state.values.subject1Id}>
					{#snippet children(subject1Id)}
						<addDifferentWeekForm.Field
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
										addDifferentWeekForm.setFieldValue('exam1Id', '');
									}}
									disabled={!subject1Id}
								/>
							{/snippet}
						</addDifferentWeekForm.Field>
					{/snippet}
				</addDifferentWeekForm.Subscribe>

				<addDifferentWeekForm.Subscribe
					selector={(state) => ({
						grade1: state.values.grade1,
						subject1Id: state.values.subject1Id
					})}
				>
					{#snippet children(state)}
						<addDifferentWeekForm.Field
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
						</addDifferentWeekForm.Field>
					{/snippet}
				</addDifferentWeekForm.Subscribe>
			</fieldset>

			<fieldset>
				<legend>Second Exam</legend>

				<addDifferentWeekForm.Field
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
								addDifferentWeekForm.setFieldValue('grade2', '');
								addDifferentWeekForm.setFieldValue('exam2Id', '');
							}}
						/>
					{/snippet}
				</addDifferentWeekForm.Field>

				<addDifferentWeekForm.Subscribe selector={(state) => state.values.subject2Id}>
					{#snippet children(subject2Id)}
						<addDifferentWeekForm.Field
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
										addDifferentWeekForm.setFieldValue('exam2Id', '');
									}}
									disabled={!subject2Id}
								/>
							{/snippet}
						</addDifferentWeekForm.Field>
					{/snippet}
				</addDifferentWeekForm.Subscribe>

				<addDifferentWeekForm.Subscribe
					selector={(state) => ({
						grade2: state.values.grade2,
						subject2Id: state.values.subject2Id
					})}
				>
					{#snippet children(state)}
						<addDifferentWeekForm.Field
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
						</addDifferentWeekForm.Field>
					{/snippet}
				</addDifferentWeekForm.Subscribe>
			</fieldset>
		</form>

		<footer>
			<addDifferentWeekForm.Subscribe
				selector={(state) => ({
					canSubmit: state.canSubmit,
					isSubmitting: state.isSubmitting,
					isPristine: state.isPristine
				})}
			>
				{#snippet children(state)}
					<Button
						form="add-different-week-constraint-form"
						type="submit"
						variant="filled"
						disabled={!state.canSubmit || state.isSubmitting || state.isPristine}
					>
						Add
					</Button>
				{/snippet}
			</addDifferentWeekForm.Subscribe>
		</footer>

		{#if differentWeekExamPairs.length === 0}
			<p>No different-week constraints yet.</p>
		{:else}
			<ul>
				{#each differentWeekExamPairs as pair (toPairKey(pair.exam1Id, pair.exam2Id))}
					<li>
						<span>{formatExamLabel(pair.firstExam)} — {formatExamLabel(pair.secondExam)}</span>
						<Button
							variant="text"
							color="error"
							onclick={() => removeDifferentWeekConstraint(pair.exam1Id, pair.exam2Id)}
						>
							Delete
						</Button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</Card>
