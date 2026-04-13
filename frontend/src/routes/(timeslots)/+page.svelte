<script lang="ts">
	import { db } from '$lib/db';
	import { sessionTimeConfig, timeslot } from '$lib/db/schema';
	import EnhancedDateField from '$lib/EnhancedDatePicker.svelte';
	import { dateKeyUTC } from '$lib/dateKeys';
	import { asc, eq, inArray } from 'drizzle-orm';
	import { createForm } from '@tanstack/svelte-form';
	import { pipe, date } from 'valibot';
	import { Button, Icon } from 'm3-svelte';
	import SaveIcon from '@ktibow/iconset-material-symbols/save-rounded';
	import { getDatesBetween } from '../subjects/forms';
	import { Temporal } from '@js-temporal/polyfill';
	import { SvelteMap } from 'svelte/reactivity';

	/** AI-generated (GPT-5.3-codex). */
	async function getTimeslots() {
		return db.query.timeslot.findMany({
			orderBy: {
				date: 'asc',
				slot: 'asc'
			}
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getInitialStartDate() {
		return db.query.timeslot.findFirst({
			orderBy: { date: 'asc' },
			columns: { date: true }
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getInitialEndDate() {
		return db.query.timeslot.findFirst({
			orderBy: { date: 'desc' },
			columns: { date: true }
		});
	}

	/** AI-generated (GPT-5.3-codex). */
	async function getSessionTimeConfig(slot: number) {
		return db.query.sessionTimeConfig.findFirst({
			where: { slot }
		});
	}

	let allTimeslots = $state(await getTimeslots());
	const initialStartDate = await getInitialStartDate();
	const initialEndDate = await getInitialEndDate();
	const initialSession1Config = await getSessionTimeConfig(0);
	const initialSession2Config = await getSessionTimeConfig(1);

	let sessionReadingStarts = $state({
		slot0: initialSession1Config?.readingStartTime ?? new Temporal.PlainTime(7, 45, 0),
		slot1: initialSession2Config?.readingStartTime ?? new Temporal.PlainTime(11, 45, 0)
	});

	const examPeriodForm = createForm(() => ({
		defaultValues: {
			startDate: initialStartDate?.date ?? new Date(),
			endDate: initialEndDate?.date ?? new Date()
		},
		onSubmit: async ({ value, formApi }) => {
			const allDays = getDatesBetween(value.startDate, value.endDate);
			const nextTimeslots = allDays.flatMap((date) => [
				{ date, slot: 0, startTime: sessionReadingStarts.slot0 },
				{ date, slot: 1, startTime: sessionReadingStarts.slot1 }
			]);

			// TODO: replace this with a more efficient bulk upsert and delete
			await db.transaction(async (tx) => {
				// eslint-disable-next-line drizzle/enforce-delete-with-where
				await tx.delete(timeslot);
				await tx.insert(timeslot).values(nextTimeslots).returning({ id: timeslot.id });
			});

			await refreshTimeslots();
			formApi.reset(value);
		}
	}));

	const sessionTimesForm = createForm(() => ({
		defaultValues: {
			session1ReadingStartTime: sessionReadingStarts.slot0,
			session1ExamStartTime:
				initialSession1Config?.examStartTime ?? new Temporal.PlainTime(8, 0, 0),
			session2ReadingStartTime: sessionReadingStarts.slot1,
			session2ExamStartTime:
				initialSession2Config?.examStartTime ?? new Temporal.PlainTime(12, 0, 0)
		},
		onSubmit: async ({ value, formApi }) => {
			await db.transaction(async (tx) => {
				await tx
					.insert(sessionTimeConfig)
					.values({
						slot: 0,
						readingStartTime: value.session1ReadingStartTime,
						examStartTime: value.session1ExamStartTime
					})
					.onConflictDoUpdate({
						target: sessionTimeConfig.slot,
						set: {
							readingStartTime: value.session1ReadingStartTime,
							examStartTime: value.session1ExamStartTime
						}
					});

				await tx
					.insert(sessionTimeConfig)
					.values({
						slot: 1,
						readingStartTime: value.session2ReadingStartTime,
						examStartTime: value.session2ExamStartTime
					})
					.onConflictDoUpdate({
						target: sessionTimeConfig.slot,
						set: {
							readingStartTime: value.session2ReadingStartTime,
							examStartTime: value.session2ExamStartTime
						}
					});
			});

			sessionReadingStarts = {
				slot0: value.session1ReadingStartTime,
				slot1: value.session2ReadingStartTime
			};
			formApi.reset(value);
		}
	}));

	/** AI-generated (GPT-5.3-codex). */
	function syncExamPeriodFromTimeslots(rows: (typeof allTimeslots)[number][]) {
		if (rows.length === 0) return;

		// safety: we checked if rows is empty above
		let startDate = rows[0]!.date;
		let endDate = rows[0]!.date;

		for (const row of rows) {
			if (row.date < startDate) startDate = row.date;
			if (row.date > endDate) endDate = row.date;
		}

		examPeriodForm.reset({
			startDate: new Date(startDate),
			endDate: new Date(endDate)
		});
	}
	const byDate = $derived.by(() => {
		const map = new SvelteMap<string, { date: Date; slots: (typeof allTimeslots)[number][] }>();

		for (const row of allTimeslots) {
			const key = dateKeyUTC(row.date);
			const existing = map.get(key);
			if (existing) {
				existing.slots.push(row);
			} else {
				map.set(key, { date: row.date, slots: [row] });
			}
		}

		for (const day of map.values()) {
			day.slots.sort((a, b) => Number(a.slot) - Number(b.slot));
		}

		return map;
	});

	const startingDate =
		(
			await db.select({ date: timeslot.date }).from(timeslot).orderBy(asc(timeslot.date)).limit(1)
		)[0]?.date ?? new Date();

	let focusedMonth = $state(startingDate.getUTCMonth());
	let focusedYear = $state(startingDate.getUTCFullYear());

	/** AI-generated (GPT-5.3-codex). */
	async function refreshTimeslots() {
		allTimeslots = await getTimeslots();
		syncExamPeriodFromTimeslots(allTimeslots);
	}

	/** AI-generated (GPT-5.3-codex). */
	function monthName(month: number) {
		const months = [
			'January',
			'February',
			'March',
			'April',
			'May',
			'June',
			'July',
			'August',
			'September'
		];
		const rest = ['October', 'November', 'December'];
		return [...months, ...rest][month] ?? 'Unknown';
	}

	/** AI-generated (GPT-5.3-codex). */
	function dayHasSlot(slots: (typeof allTimeslots)[number][], slotNumber: number): boolean {
		return slots.some((slot) => Number(slot.slot) === slotNumber);
	}

	/** AI-generated (GPT-5.3-codex). */
	function slotByNumber(slots: (typeof allTimeslots)[number][], slotNumber: number) {
		return slots.find((slot) => Number(slot.slot) === slotNumber) ?? null;
	}

	/** AI-generated (GPT-5.3-codex). */
	function slotId(slots: (typeof allTimeslots)[number][], slotNumber: number): number | null {
		return slotByNumber(slots, slotNumber)?.id ?? null;
	}

	/** AI-generated (GPT-5.3-codex). */
	function moveMonth(delta: number) {
		const next = new Date(Date.UTC(focusedYear, focusedMonth + delta, 1));
		focusedYear = next.getUTCFullYear();
		focusedMonth = next.getUTCMonth();
	}

	const monthTitle = $derived(`${monthName(focusedMonth)} ${focusedYear}`);

	const firstOfMonth = $derived(new Date(Date.UTC(focusedYear, focusedMonth, 1)));
	const startOffset = $derived((firstOfMonth.getUTCDay() + 6) % 7);

	const visibleCells = $derived.by(() => {
		return Array.from({ length: 42 }, (_, index) => {
			const date = new Date(Date.UTC(focusedYear, focusedMonth, index - startOffset + 1));
			return {
				date,
				dayNumber: date.getUTCDate(),
				inMonth: date.getUTCMonth() === focusedMonth,
				slots: byDate.get(dateKeyUTC(date))?.slots ?? []
			};
		});
	});

	/** AI-generated (GPT-5.3-codex). */
	async function removeTimeslot(timeslotId: number) {
		await db.delete(timeslot).where(eq(timeslot.id, timeslotId));
		await refreshTimeslots();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function removeDate(date: Date) {
		const key = dateKeyUTC(date);
		const ids = allTimeslots.filter((row) => dateKeyUTC(row.date) === key).map((row) => row.id);
		if (ids.length === 0) return;

		await db.delete(timeslot).where(inArray(timeslot.id, ids));
		await refreshTimeslots();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function removeWeekends() {
		const weekendIds = allTimeslots
			.filter((row) => {
				const day = row.date.getUTCDay();
				return day === 0 || day === 6;
			})
			.map((row) => row.id);

		if (weekendIds.length === 0) return;
		await db.delete(timeslot).where(inArray(timeslot.id, weekendIds));
		await refreshTimeslots();
	}

	/** AI-generated (GPT-5.3-codex). */
	async function addSlotForDate(date: Date, slotNumber: number) {
		const key = dateKeyUTC(date);
		const slots = byDate.get(key)?.slots ?? [];
		if (dayHasSlot(slots, slotNumber)) return;

		await db.insert(timeslot).values({
			date,
			slot: slotNumber
		});
		await refreshTimeslots();
	}
</script>

<header>
	<div>
		<h1>Timeslots</h1>
		<h2>Set exam period, session times, and available slots.</h2>
	</div>
</header>

<main>
	<section aria-labelledby="configuration-heading">
		<h3 id="configuration-heading">Configuration</h3>

		<div class="configuration-grid">
			<form
				id="timeslots-config-form"
				onsubmit={(event) => {
					event.preventDefault();
					event.stopPropagation();
					examPeriodForm.handleSubmit();
				}}
			>
				<fieldset>
					<legend>Exam period</legend>
					<div class="field-grid">
						<examPeriodForm.Field name="startDate" validators={{ onChange: pipe(date()) }}>
							{#snippet children(field)}
								<EnhancedDateField
									{field}
									variant="outlined"
									label="Start date"
									datePickerTitle="Select starting date"
									required
								/>
							{/snippet}
						</examPeriodForm.Field>

						<examPeriodForm.Field
							name="endDate"
							validators={{
								onBlur: pipe(date()),
								onChangeListenTo: ['startDate'],
								onChange: ({ value, fieldApi }) => {
									const startDate = fieldApi.form.getFieldValue('startDate');
									if (!value || !startDate) return undefined;
									if (value <= startDate) return 'End date must be after the start date';
									return undefined;
								}
							}}
						>
							{#snippet children(field)}
								<EnhancedDateField
									{field}
									variant="outlined"
									label="Ending date"
									datePickerTitle="Select ending date"
									required
								/>
							{/snippet}
						</examPeriodForm.Field>
					</div>
				</fieldset>
				<footer>
					<examPeriodForm.Subscribe
						selector={(state) => ({
							isDirty: state.isDirty,
							canSubmit: state.canSubmit,
							isSubmitting: state.isSubmitting
						})}
					>
						{#snippet children(state)}
							{@const enabled = state.isDirty && state.canSubmit && !state.isSubmitting}
							<Button iconType="left" type="submit" disabled={!enabled}>
								<Icon icon={SaveIcon} />
								Generate timeslots
							</Button>
						{/snippet}
					</examPeriodForm.Subscribe>
				</footer>
			</form>

			<form
				id="session-times-form"
				onsubmit={(event) => {
					event.preventDefault();
					event.stopPropagation();
					sessionTimesForm.handleSubmit();
				}}
			>
				<fieldset>
					<legend>Session times</legend>
					<div class="field-grid">
						<label>
							Session 1 reading start
							<sessionTimesForm.Field name="session1ReadingStartTime">
								{#snippet children(field)}
									<input
										type="time"
										step="60"
										value={field.state.value.toString({ smallestUnit: 'minute' })}
										onchange={(event) =>
											field.handleChange(
												Temporal.PlainTime.from((event.currentTarget as HTMLInputElement).value)
											)}
									/>
								{/snippet}
							</sessionTimesForm.Field>
						</label>

						<label>
							Session 1 exam start
							<sessionTimesForm.Field name="session1ExamStartTime">
								{#snippet children(field)}
									<input
										type="time"
										step="60"
										value={field.state.value.toString({ smallestUnit: 'minute' })}
										onchange={(event) =>
											field.handleChange(
												Temporal.PlainTime.from((event.currentTarget as HTMLInputElement).value)
											)}
									/>
								{/snippet}
							</sessionTimesForm.Field>
						</label>

						<label>
							Session 2 reading start
							<sessionTimesForm.Field name="session2ReadingStartTime">
								{#snippet children(field)}
									<input
										type="time"
										step="60"
										value={field.state.value.toString({ smallestUnit: 'minute' })}
										onchange={(event) =>
											field.handleChange(
												Temporal.PlainTime.from((event.currentTarget as HTMLInputElement).value)
											)}
									/>
								{/snippet}
							</sessionTimesForm.Field>
						</label>

						<label>
							Session 2 exam start
							<sessionTimesForm.Field name="session2ExamStartTime">
								{#snippet children(field)}
									<input
										type="time"
										step="60"
										value={field.state.value.toString({ smallestUnit: 'minute' })}
										onchange={(event) =>
											field.handleChange(
												Temporal.PlainTime.from((event.currentTarget as HTMLInputElement).value)
											)}
									/>
								{/snippet}
							</sessionTimesForm.Field>
						</label>
					</div>
				</fieldset>
				<footer>
					<sessionTimesForm.Subscribe
						selector={(state) => ({
							isDirty: state.isDirty,
							canSubmit: state.canSubmit,
							isSubmitting: state.isSubmitting
						})}
					>
						{#snippet children(state)}
							{@const enabled = state.isDirty && state.canSubmit && !state.isSubmitting}
							<Button iconType="left" type="submit" disabled={!enabled}>
								<Icon icon={SaveIcon} />
								Save Session Times
							</Button>
						{/snippet}
					</sessionTimesForm.Subscribe>
				</footer>
			</form>
		</div>
	</section>

	{#if allTimeslots.length === 0}
		<section>
			<p>No timeslots available yet. Set an exam period above and generate timeslots.</p>
		</section>
	{:else}
		<section aria-labelledby="available-timeslots-heading">
			<header>
				<h3 id="available-timeslots-heading">Available timeslots</h3>
				<Button
					variant="tonal"
					type="button"
					onclick={() => {
						void removeWeekends();
					}}
				>
					Remove weekends
				</Button>
			</header>

			<nav class="month-header" aria-label="Calendar month navigation">
				<button type="button" class="month-nav" onclick={() => moveMonth(-1)}> Prev </button>
				<p class="month-title">{monthTitle}</p>
				<button type="button" class="month-nav" onclick={() => moveMonth(1)}> Next </button>
			</nav>

			<ol class="weekday-row" aria-hidden="true">
				{#each ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] as weekday (weekday)}
					<li>{weekday}</li>
				{/each}
			</ol>

			<div class="calendar-grid">
				{#each visibleCells as cell (cell.date.getTime())}
					<article class="day-cell" class:outside={!cell.inMonth}>
						<header>
							<span>{cell.dayNumber}</span>
							{#if cell.inMonth && cell.slots.length > 0}
								<button
									type="button"
									class="danger-button"
									title="Remove date"
									onclick={() => void removeDate(cell.date)}
								>
									Remove day
								</button>
							{/if}
						</header>

						{#if cell.inMonth}
							<ul class="slot-stack">
								{#each [0, 1] as slotNumber (slotNumber)}
									<li class="slot-row">
										<h4>Slot {slotNumber + 1}</h4>
										{#if dayHasSlot(cell.slots, slotNumber)}
											<div class="slot-editor">
												<span class="available-tag">Available</span>
												<button
													type="button"
													class="danger-button"
													onclick={() => {
														const id = slotId(cell.slots, slotNumber);
														if (id === null) return;
														void removeTimeslot(id);
													}}
												>
													Remove
												</button>
											</div>
										{:else}
											<button
												type="button"
												class="add-button"
												onclick={() => void addSlotForDate(cell.date, slotNumber)}
											>
												Add Slot {slotNumber + 1}
											</button>
										{/if}
									</li>
								{/each}
							</ul>
						{:else if cell.slots.length === 0}
							<p>No slots</p>
						{/if}
					</article>
				{/each}
			</div>
		</section>
	{/if}
</main>

<style>
	section {
		display: grid;
		gap: 0.75rem;
		padding: 0.75rem;
	}

	form {
		display: grid;
		grid-template-rows: 1fr auto;
		gap: 1rem;
		height: 100%;
	}

	section > h3 {
		margin: 0;
		font-size: 1.05rem;
		font-weight: 700;
	}

	.configuration-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 1rem;
	}

	.field-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 0.75rem 1rem;
	}

	form > footer {
		display: flex;
		justify-content: flex-end;
		align-items: center;
		margin-top: 0.25rem;
	}

	form > footer :global(button) {
		min-width: 12rem;
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
	}

	label {
		display: grid;
		gap: 0.25rem;
		color: var(--m3c-on-surface-variant);
		font-size: 0.9rem;
	}

	input[type='time'] {
		padding: 0.4rem 0.5rem;
		border: 1px solid var(--m3c-outline-variant);
		border-radius: 0.5rem;
		background: var(--m3c-surface);
		color: var(--m3c-on-surface);
	}

	.month-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.75rem;
	}

	section > header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.75rem;
	}

	section > header > h3 {
		margin: 0;
		@apply --m3-title-large;
		font-weight: 700;
		color: var(--m3c-on-surface);
	}

	section > header :global(button) {
		--md-filled-tonal-button-container-shape: 0.75rem;
	}

	.month-nav {
		border: 1px solid var(--m3c-outline-variant);
		border-radius: 999px;
		background: var(--m3c-surface);
		color: var(--m3c-on-surface);
		padding: 0.35rem 0.8rem;
	}

	.month-title {
		margin: 0;
		@apply --m3-title-large;
		color: var(--m3c-on-surface);
	}

	.weekday-row {
		display: grid;
		grid-template-columns: repeat(7, minmax(0, 1fr));
		gap: 0.5rem;
		list-style: none;
		padding: 0;
	}

	.weekday-row > li {
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--m3c-on-surface-variant);
		text-align: center;
	}

	.calendar-grid {
		display: grid;
		grid-template-columns: repeat(7, minmax(0, 1fr));
		gap: 0.5rem;
	}

	.day-cell {
		display: grid;
		align-content: start;
		gap: 0.45rem;
		min-height: 10.5rem;
		padding: 0.5rem;
		border-radius: 0.75rem;
		background: var(--m3c-surface-container-high);
	}

	.day-cell.outside {
		opacity: 0.45;
	}

	.day-cell > header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-weight: 700;
	}

	.danger-button {
		border: 1px solid color-mix(in oklch, var(--m3c-error) 35%, transparent);
		background: color-mix(in oklch, var(--m3c-error) 15%, transparent);
		border-radius: 0.5rem;
		color: var(--m3c-error);
		font-size: 0.72rem;
		font-weight: 700;
		cursor: pointer;
		padding: 0.15rem 0.4rem;
	}

	.slot-stack {
		display: grid;
		gap: 0.45rem;
		list-style: none;
		padding: 0;
	}

	.slot-row {
		display: grid;
		gap: 0.3rem;
	}

	h4 {
		margin: 0;
		font-size: 0.74rem;
		font-weight: 600;
		color: var(--m3c-on-surface-variant);
	}

	.slot-editor {
		display: grid;
		gap: 0.3rem;
	}

	.add-button {
		border: 1px dashed var(--m3c-primary);
		background: color-mix(in oklch, var(--m3c-primary) 12%, transparent);
		color: var(--m3c-primary);
		border-radius: 0.45rem;
		font-size: 0.72rem;
		padding: 0.18rem 0.35rem;
		cursor: pointer;
		inline-size: 100%;
		min-height: 1.75rem;
	}

	.add-button:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.day-cell > p {
		margin: 0;
		font-size: 0.75rem;
		color: var(--m3c-on-surface-variant);
	}

	.available-tag {
		font-size: 0.76rem;
		font-weight: 600;
		color: var(--m3c-on-surface-variant);
	}

	@media (max-width: 70rem) {
		.configuration-grid {
			grid-template-columns: 1fr;
		}

		.field-grid {
			grid-template-columns: 1fr;
		}

		.calendar-grid,
		.weekday-row {
			grid-template-columns: repeat(4, minmax(0, 1fr));
		}
	}

	@media (max-width: 42rem) {
		section > header {
			flex-direction: column;
			align-items: stretch;
		}

		.calendar-grid,
		.weekday-row {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}
</style>
