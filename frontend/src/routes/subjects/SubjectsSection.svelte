<script lang="ts">
	import { Button, Chip, Icon } from 'm3-svelte';
	import PlusIcon from '@ktibow/iconset-material-symbols/add-rounded';
	import { formatExamLabel } from '$lib/examDisplay';
	import type { exam } from '$lib/db/schema';

	type SubjectRow = {
		id: number;
		name: string;
		subjectGradesSubjectId: Array<{
			grade: number;
			exams: Array<typeof exam.$inferSelect>;
		}>;
	};

	let {
		subjects,
		onEditSubject,
		onDeleteSubject,
		onAddGrade,
		onDeleteGrade,
		onAddExam,
		onEditExam
	}: {
		subjects: SubjectRow[];
		onEditSubject: (subjectId: number, name: string) => void;
		onDeleteSubject: (subjectId: number) => void;
		onAddGrade: (subjectId: number) => void;
		onDeleteGrade: (subjectId: number, grade: number) => void;
		onAddExam: (subjectId: number, grade: number, paper: number) => void;
		onEditExam: (examRow: typeof exam.$inferSelect) => void;
	} = $props();
</script>

<section>
	<table>
		<thead>
			<tr>
				<th>Subject / Grade</th>
				<th>Exams</th>
				<th>Actions</th>
			</tr>
		</thead>

		<tbody>
			{#each subjects as row (row.id)}
				<tr>
					<td colspan="2">{row.name}</td>
					<td>
						<Button onclick={() => onEditSubject(row.id, row.name)}>Edit</Button>
						<Button variant="tonal" color="error" onclick={() => onDeleteSubject(row.id)}>
							Delete
						</Button>
					</td>
				</tr>
				{#each row.subjectGradesSubjectId as grade (grade.grade)}
					<tr class="indent">
						<td>{row.name} Grade {grade.grade}</td>
						<td>
							<ul class="chips">
								{#each grade.exams as examRow (examRow.id)}
									<li>
										<Chip variant="input" onclick={() => onEditExam(examRow)}>
											{formatExamLabel({ ...examRow, subject: { name: row.name } })}
										</Chip>
									</li>
								{/each}

								<li>
									<Chip
										variant="assist"
										icon={PlusIcon}
										onclick={() => onAddExam(row.id, grade.grade, grade.exams.length + 1)}
									>
										Add Exam
									</Chip>
								</li>
							</ul>
						</td>
						<td>
							<Button
								variant="tonal"
								color="error"
								onclick={() => onDeleteGrade(row.id, grade.grade)}
							>
								Delete
							</Button>
						</td>
					</tr>
				{/each}
				<tr class="indent">
					<td>
						<Button onclick={() => onAddGrade(row.id)} iconType="left" variant="outlined">
							<Icon icon={PlusIcon} />
							Add Grade
						</Button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</section>

<style>
	section {
		padding: 1rem;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		border-radius: var(--m3-shape-medium);
		overflow: hidden;
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

			&:not(.indent) {
				background-color: var(--m3c-surface-container);

				& td:first-child {
					font-weight: 600;
					box-shadow: inset 3px 0 0 var(--m3c-primary);
					padding-left: calc(1rem - 3px);
				}
			}

			&.indent {
				background-color: var(--m3c-surface-container-low);

				& td:first-child {
					padding-left: 2.25rem;
					color: var(--m3c-on-surface-variant);
					white-space: nowrap;
				}

				& td:nth-child(2) {
					padding-block: 0.5rem;

					& .chips {
						display: flex;
						flex-wrap: wrap;
						gap: 0.5rem;
						align-items: center;
						list-style: none;
						padding: 0;
					}
				}

				&:has(td:only-child) {
					background-color: transparent;
					border-bottom: none;

					& td {
						padding-block: 0.375rem 0.625rem;
						padding-left: 2.25rem;
					}

					& + tr:not(.indent) {
						border-top: 1px solid var(--m3c-outline-variant);
					}
				}
			}
		}

		& td {
			@apply --m3-body-medium;
			height: 100%;
			color: var(--m3c-on-surface);
			padding: 0.75rem 1rem;
			vertical-align: middle;
		}
	}
</style>
