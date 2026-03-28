<script lang="ts">
	import { db } from '$lib/db';
	import { sameDayExam } from '$lib/db/schema';
	import { and, eq } from 'drizzle-orm';
	import { Button, Card } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import { pipe, string, nonEmpty } from 'valibot';
	import { formatExamLabel } from '$lib/examDisplay';
	import EnhancedSelect from '$lib/EnhancedSelect.svelte';

	let { onCountChange }: { onCountChange?: (count: number) => void } = $props();

	/** AI-generated (GPT-5.3-codex). */
	async function getSameDayExams() {
		return await db.query.sameDayExam.findMany({
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
				firstSlotExamId: 'asc',
				secondSlotExamId: 'asc'
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
	async function refreshSameDayExams() {
		sameDayExams = await getSameDayExams();
	}

	let sameDayExams = $state(await getSameDayExams());
	const subjectOptions = await getSubjectOptions();

	const sameDayPairKeys = $derived(
		new Set(sameDayExams.map((pair) => toPairKey(pair.firstSlotExamId, pair.secondSlotExamId)))
	);

	const addSameDayForm = createForm(() => ({
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
				if (sameDayPairKeys.has(pairKey)) return 'This pair already exists.';

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
				.insert(sameDayExam)
				.values({ firstSlotExamId, secondSlotExamId })
				.onConflictDoNothing();

			await refreshSameDayExams();
			formApi.reset();
		}
	}));

	/** AI-generated (GPT-5.3-codex). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		addSameDayForm.handleSubmit();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function removeSameDayConstraint(firstSlotExamId: number, secondSlotExamId: number) {
		await db
			.delete(sameDayExam)
			.where(
				and(
					eq(sameDayExam.firstSlotExamId, firstSlotExamId),
					eq(sameDayExam.secondSlotExamId, secondSlotExamId)
				)
			);
		await refreshSameDayExams();
	}

	$effect(() => {
		onCountChange?.(sameDayExams.length);
	});
</script>

<Card variant="filled">
	<section data-constraint-card>
		<header>
			<h2>Same Day Exams</h2>
			<p>These two exams must happen on the same date.</p>
		</header>

		<form id="add-same-day-constraint-form" onsubmit={handleFormSubmit}>
			<fieldset>
				<legend>First Exam</legend>

				<addSameDayForm.Field
					name="subject1Id"
					validators={{
						onChange: pipe(string(), nonEmpty('Pick a subject for the first exam.'))
					}}
				>
					{#snippet children(field)}
						<EnhancedSelect
							{field}
							variant="outlined"
							label="Subject A"
							options={subjectOptions}
							onValueChange={() => {
								addSameDayForm.setFieldValue('grade1', '');
								addSameDayForm.setFieldValue('exam1Id', '');
							}}
						/>
					{/snippet}
				</addSameDayForm.Field>

				<addSameDayForm.Subscribe selector={(state) => state.values.subject1Id}>
					{#snippet children(subject1Id)}
						<addSameDayForm.Field
							name="grade1"
							validators={{
								onChange: pipe(string(), nonEmpty('Pick a grade for the first exam.'))
							}}
						>
							{#snippet children(field)}
								<EnhancedSelect
									{field}
									variant="outlined"
									label="Grade 1"
									options={await getGradeOptionsForSelectedSubject(subject1Id)}
									onValueChange={() => {
										addSameDayForm.setFieldValue('exam1Id', '');
									}}
									disabled={!subject1Id}
								/>
							{/snippet}
						</addSameDayForm.Field>
					{/snippet}
				</addSameDayForm.Subscribe>

				<addSameDayForm.Subscribe
					selector={(state) => ({
						grade1: state.values.grade1,
						subject1Id: state.values.subject1Id
					})}
				>
					{#snippet children(state)}
						<addSameDayForm.Field
							name="exam1Id"
							validators={{
								onChange: pipe(string(), nonEmpty('Pick paper for the first exam.'))
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
						</addSameDayForm.Field>
					{/snippet}
				</addSameDayForm.Subscribe>
			</fieldset>

			<fieldset>
				<legend>Second Exam</legend>

				<addSameDayForm.Field
					name="subject2Id"
					validators={{
						onChange: pipe(string(), nonEmpty('Pick a subject for the second exam.'))
					}}
				>
					{#snippet children(field)}
						<EnhancedSelect
							{field}
							variant="outlined"
							label="Subject 2"
							options={subjectOptions}
							onValueChange={() => {
								addSameDayForm.setFieldValue('grade2', '');
								addSameDayForm.setFieldValue('exam2Id', '');
							}}
						/>
					{/snippet}
				</addSameDayForm.Field>

				<addSameDayForm.Subscribe selector={(state) => state.values.subject2Id}>
					{#snippet children(subject2Id)}
						<addSameDayForm.Field
							name="grade2"
							validators={{
								onChange: pipe(string(), nonEmpty('Pick a grade for the second exam.'))
							}}
						>
							{#snippet children(field)}
								<EnhancedSelect
									{field}
									variant="outlined"
									label="Grade 2"
									options={await getGradeOptionsForSelectedSubject(subject2Id)}
									onValueChange={() => {
										addSameDayForm.setFieldValue('exam2Id', '');
									}}
									disabled={!subject2Id}
								/>
							{/snippet}
						</addSameDayForm.Field>
					{/snippet}
				</addSameDayForm.Subscribe>

				<addSameDayForm.Subscribe
					selector={(state) => ({
						grade2: state.values.grade2,
						subject2Id: state.values.subject2Id
					})}
				>
					{#snippet children(state)}
						<addSameDayForm.Field
							name="exam2Id"
							validators={{
								onChange: pipe(string(), nonEmpty('Pick paper for the second exam.'))
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
						</addSameDayForm.Field>
					{/snippet}
				</addSameDayForm.Subscribe>
			</fieldset>
		</form>

		<footer>
			<addSameDayForm.Subscribe
				selector={(state) => ({
					canSubmit: state.canSubmit,
					isSubmitting: state.isSubmitting,
					isPristine: state.isPristine
				})}
			>
				{#snippet children(state)}
					<Button
						form="add-same-day-constraint-form"
						type="submit"
						variant="filled"
						disabled={!state.canSubmit || state.isSubmitting || state.isPristine}
					>
						Add
					</Button>
				{/snippet}
			</addSameDayForm.Subscribe>
		</footer>

		{#if sameDayExams.length === 0}
			<p>No same-day constraints yet.</p>
		{:else}
			<ul>
				{#each sameDayExams as pair ((pair.firstSlotExamId, pair.secondSlotExamId))}
					<li>
						<span>{formatExamLabel(pair.firstExam)} — {formatExamLabel(pair.secondExam)}</span>
						<Button
							variant="text"
							color="error"
							onclick={() => removeSameDayConstraint(pair.firstSlotExamId, pair.secondSlotExamId)}
						>
							Delete
						</Button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</Card>
