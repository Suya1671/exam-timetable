<script lang="ts">
    import { Button, Chip, Dialog, Icon } from "m3-svelte";
    import EnhancedDateField from "$lib/EnhancedDatePicker.svelte";
    import SaveIcon from "@ktibow/iconset-material-symbols/save-rounded";
    import { createForm } from "@tanstack/svelte-form";
    import {
        pipe,
        string,
        nonEmpty,
        number,
        minValue,
        maxValue,
        integer,
        date,
    } from "valibot";
    import PlusIcon from "@ktibow/iconset-material-symbols/add-rounded";
    import EnhancedTextInput from "$lib/EnhancedTextInput.svelte";
    import TimeslotPicker from "$lib/TimeslotPicker.svelte";
    import { db } from "$lib/db";
    import {
        exam,
        examAllowedTimeslot,
        examDeniedTimeslot,
        subject,
        subjectGrade,
        timeslot,
    } from "$lib/db/schema";
    import { and, eq } from "drizzle-orm";
    import { SvelteSet } from "svelte/reactivity";

    async function getSubjects() {
        return db.query.subject.findMany({
            with: {
                subjectGradesSubjectId: {
                    with: {
                        exams: true,
                    },
                },
            },
        });
    }

    let data = $state(await getSubjects());
    let allTimeslots = $state(
        await db.select().from(timeslot).orderBy(timeslot.date, timeslot.slot),
    );

    const addExamAllowedIds = new SvelteSet<number>();
    const addExamDeniedIds = new SvelteSet<number>();
    const editExamAllowedIds = new SvelteSet<number>();
    const editExamDeniedIds = new SvelteSet<number>();

    async function refreshTimeslots() {
        allTimeslots = await db
            .select()
            .from(timeslot)
            .orderBy(timeslot.date, timeslot.slot);
    }

    async function loadExamTimeslotRestrictions(examId: number) {
        const [allowedRows, deniedRows] = await db.batch([
            db
                .select({ timeslotId: examAllowedTimeslot.timeslotId })
                .from(examAllowedTimeslot)
                .where(eq(examAllowedTimeslot.examId, examId)),
            db
                .select({ timeslotId: examDeniedTimeslot.timeslotId })
                .from(examDeniedTimeslot)
                .where(eq(examDeniedTimeslot.examId, examId)),
        ]);

        editExamAllowedIds.clear();
        for (const row of allowedRows) {
            editExamAllowedIds.add(row.timeslotId);
        }

        editExamDeniedIds.clear();
        for (const row of deniedRows) {
            editExamDeniedIds.add(row.timeslotId);
        }
    }

    async function replaceExamTimeslotRestrictions(
        examId: number,
        allowedIds: Set<number>,
        deniedIds: Set<number>,
    ) {
        await db.transaction(async (tx) => {
            await tx
                .delete(examAllowedTimeslot)
                .where(eq(examAllowedTimeslot.examId, examId));
            await tx
                .delete(examDeniedTimeslot)
                .where(eq(examDeniedTimeslot.examId, examId));

            if (allowedIds.size > 0) {
                await tx.insert(examAllowedTimeslot).values(
                    [...allowedIds].map((timeslotId) => ({
                        examId,
                        timeslotId,
                    })),
                );
            }

            if (deniedIds.size > 0) {
                await tx.insert(examDeniedTimeslot).values(
                    [...deniedIds].map((timeslotId) => ({
                        examId,
                        timeslotId,
                    })),
                );
            }
        });
    }

    function getDatesBetween(start: Date, end: Date): Date[] {
        const startUTC = Date.UTC(
            start.getFullYear(),
            start.getMonth(),
            start.getDate(),
        );

        const endUTC = Date.UTC(
            end.getFullYear(),
            end.getMonth(),
            end.getDate(),
        );

        if (endUTC < startUTC) {
            throw new Error("End date must be after start date");
        }

        const dayMs = 24 * 60 * 60 * 1000;
        const length = Math.floor((endUTC - startUTC) / dayMs) + 1;

        return Array.from({ length }, (_, i) => new Date(startUTC + i * dayMs));
    }

    const initialStartDate = await db.query.timeslot.findFirst({
        orderBy: {
            date: "asc",
        },
        columns: {
            date: true,
        },
    });

    const initialEndDate = await db.query.timeslot.findFirst({
        orderBy: {
            date: "desc",
        },
        columns: {
            date: true,
        },
    });

    const informationForm = createForm(() => ({
        defaultValues: {
            startDate: initialStartDate?.date ?? new Date(),
            endDate: initialEndDate?.date ?? new Date(),
        },

        onSubmit: async ({ value, formApi }) => {
            const allDays = getDatesBetween(value.startDate, value.endDate);
            const allTimeslots: (typeof timeslot.$inferInsert)[] =
                allDays.flatMap((date) => [
                    {
                        date,
                        slot: 0,
                    },
                    {
                        date,
                        slot: 1,
                    },
                ]);

            await db.transaction(async (tx) => {
                await tx.delete(timeslot);
                await tx
                    .insert(timeslot)
                    .values(allTimeslots)
                    .returning({ id: timeslot.id });
            });

            await refreshTimeslots();

            formApi.reset(value);
        },
    }));

    type ActiveDialog =
        | { type: "addSubject" }
        | { type: "editSubject"; subjectId: number; name: string }
        | { type: "addGrade"; subjectId: number }
        | { type: "addExam"; subjectId: number; grade: number; paper: number }
        | {
              type: "editExam";
              exam: typeof exam.$inferSelect;
          }
        | null;

    let activeDialog = $state<ActiveDialog>(null);

    function getDialogOpen(type: NonNullable<ActiveDialog>["type"]) {
        return () => activeDialog?.type === type;
    }

    function setDialogOpen(type: NonNullable<ActiveDialog>["type"]) {
        return (open: boolean) => {
            if (!open && activeDialog?.type === type) {
                activeDialog = null;
            }
        };
    }

    function handleFormSubmit(
        e: SubmitEvent,
        form: { handleSubmit: () => void },
    ) {
        e.preventDefault();
        e.stopPropagation();
        form.handleSubmit();
    }

    async function refreshSubjects() {
        data = await getSubjects();
    }

    async function openDialog(dialog: Exclude<ActiveDialog, null>) {
        switch (dialog.type) {
            case "addSubject": {
                break;
            }
            case "editSubject": {
                editSubjectForm.setFieldValue("subjectId", dialog.subjectId);
                editSubjectForm.setFieldValue("name", dialog.name);
                break;
            }
            case "addGrade": {
                addGradeForm.setFieldValue("subjectId", dialog.subjectId);
                break;
            }
            case "addExam": {
                addExamForm.setFieldValue("subjectId", dialog.subjectId);
                addExamForm.setFieldValue("grade", dialog.grade);
                addExamForm.setFieldValue("paper", dialog.paper);
                addExamAllowedIds.clear();
                addExamDeniedIds.clear();
                break;
            }
            case "editExam": {
                editExamForm.setFieldValue("id", dialog.exam.id);
                editExamForm.setFieldValue(
                    "durationHours",
                    dialog.exam.durationHours,
                );
                editExamForm.setFieldValue(
                    "slotsRequired",
                    dialog.exam.slotsRequired,
                );
                editExamForm.setFieldValue("priority", dialog.exam.priority);
                break;
            }
        }

        activeDialog = dialog;

        if (dialog.type === "editExam") {
            if (
                activeDialog?.type === "editExam" &&
                activeDialog.exam.id === dialog.exam.id
            ) {
                await loadExamTimeslotRestrictions(dialog.exam.id);
            }
        }
    }

    const addSubjectForm = createForm(() => ({
        defaultValues: {
            name: "",
        },

        onSubmit: async ({ value, formApi }) => {
            await db.insert(subject).values(value);
            activeDialog = null;
            await refreshSubjects();
            formApi.reset();
        },
    }));

    const editSubjectForm = createForm(() => ({
        defaultValues: {
            subjectId: 0,
            name: "",
        },

        onSubmit: async ({ value, formApi }) => {
            await db
                .update(subject)
                .set(value)
                .where(eq(subject.id, value.subjectId));

            activeDialog = null;
            await refreshSubjects();
            formApi.reset();
        },
    }));

    const addGradeForm = createForm(() => ({
        defaultValues: {
            subjectId: 0,
            grade: 8,
        },
        onSubmit: async ({ value, formApi }) => {
            await db.insert(subjectGrade).values(value);
            activeDialog = null;
            await refreshSubjects();
            formApi.reset();
        },
    }));

    const addExamForm = createForm(() => ({
        defaultValues: {
            // filled in when opening the dialog
            subjectId: 0,
            paper: 1,
            grade: 8,
            // editable
            slotsRequired: 1,
            durationHours: 2,
            priority: 0,
        } satisfies typeof exam.$inferInsert,

        onSubmit: async ({ value, formApi }) => {
            const [createdExam] = await db
                .insert(exam)
                .values(value)
                .returning({ id: exam.id });

            if (createdExam) {
                await replaceExamTimeslotRestrictions(
                    createdExam.id,
                    addExamAllowedIds,
                    addExamDeniedIds,
                );
            }

            activeDialog = null;
            await refreshSubjects();
            addExamAllowedIds.clear();
            addExamDeniedIds.clear();
            formApi.reset();
        },
    }));

    type EditAction = "edit" | "delete";

    const editExamForm = createForm(() => ({
        defaultValues: {
            // filled in when opening the dialog
            id: 0,
            // editable
            slotsRequired: 1,
            durationHours: 2,
            priority: 0,
        },
        onSubmitMeta: {
            action: "edit" as EditAction,
        },
        onSubmit: async ({ value, formApi, meta }) => {
            switch (meta.action) {
                case "edit": {
                    await db.transaction(async (tx) => {
                        await tx
                            .update(exam)
                            .set({
                                slotsRequired: value.slotsRequired,
                                durationHours: value.durationHours,
                                priority: value.priority,
                            })
                            .where(eq(exam.id, value.id));

                        await tx
                            .delete(examAllowedTimeslot)
                            .where(eq(examAllowedTimeslot.examId, value.id));
                        await tx
                            .delete(examDeniedTimeslot)
                            .where(eq(examDeniedTimeslot.examId, value.id));

                        if (editExamAllowedIds.size > 0) {
                            await tx.insert(examAllowedTimeslot).values(
                                [...editExamAllowedIds].map((timeslotId) => ({
                                    examId: value.id,
                                    timeslotId,
                                })),
                            );
                        }

                        if (editExamDeniedIds.size > 0) {
                            await tx.insert(examDeniedTimeslot).values(
                                [...editExamDeniedIds].map((timeslotId) => ({
                                    examId: value.id,
                                    timeslotId,
                                })),
                            );
                        }
                    });
                    break;
                }
                case "delete": {
                    if (
                        !confirm(
                            "Are you sure you want to delete this exam? This action cannot be undone.",
                        )
                    ) {
                        return;
                    }

                    await db.delete(exam).where(eq(exam.id, value.id));
                    break;
                }
            }
            await refreshSubjects();
            activeDialog = null;
            editExamAllowedIds.clear();
            editExamDeniedIds.clear();
            formApi.reset();
        },
    }));

    async function deleteSubject(subjectId: number) {
        if (
            !confirm(
                "Are you sure you want to delete this subject? This action cannot be undone.",
            )
        ) {
            return;
        }

        await db.delete(subject).where(eq(subject.id, subjectId));
        await refreshSubjects();
    }

    async function deleteGrade(subjectId: number, grade: number) {
        if (
            !confirm(
                "Are you sure you want to delete this grade? This action cannot be undone.",
            )
        ) {
            return;
        }

        await db
            .delete(subjectGrade)
            .where(
                and(
                    eq(subjectGrade.subjectId, subjectId),
                    eq(subjectGrade.grade, grade),
                ),
            );
        await refreshSubjects();
    }
</script>

<header>
    <h1>Timetable information</h1>
</header>

<main>
    <section>
        <header>
            <h2>Basic timetable information</h2>

            <informationForm.Subscribe
                selector={(state) => ({
                    isDirty: state.isDirty,
                    canSubmit: state.canSubmit,
                    isSubmitting: state.isSubmitting,
                    values: state.values,
                })}
            >
                {#snippet children(state)}
                    {@const enabled =
                        state.isDirty && state.canSubmit && !state.isSubmitting}
                    <Button
                        iconType="left"
                        form="exam-period-form"
                        type="submit"
                        disabled={!enabled}
                    >
                        <Icon icon={SaveIcon} />
                        Save Changes
                    </Button>
                {/snippet}
            </informationForm.Subscribe>
        </header>

        <form
            id="exam-period-form"
            onsubmit={(e) => {
                e.preventDefault();
                e.stopPropagation();
                informationForm.handleSubmit();
            }}
        >
            <fieldset>
                <legend>Exam period</legend>
                <p>
                    Please select the starting and ending date of the exam
                    period. The timetable will be generated for the dates in
                    this range.
                </p>

                <informationForm.Field
                    name="startDate"
                    validators={{
                        onChange: pipe(date()),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedDateField
                            {field}
                            label="Start date"
                            datePickerTitle="Select starting date"
                            required
                        ></EnhancedDateField>
                    {/snippet}
                </informationForm.Field>

                <informationForm.Field
                    name="endDate"
                    validators={{
                        onBlur: pipe(date()),
                        onChangeListenTo: ["startDate"],
                        onChange: ({ value, fieldApi }) => {
                            const startDate =
                                fieldApi.form.getFieldValue("startDate");
                            if (!value || !startDate) return undefined;
                            if (value <= startDate) {
                                return "End date must be after the start date";
                            }
                            return undefined;
                        },
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedDateField
                            {field}
                            label="Ending date"
                            datePickerTitle="Select ending date"
                            required
                        ></EnhancedDateField>
                    {/snippet}
                </informationForm.Field>
            </fieldset>
        </form>
    </section>

    <section>
        <header>
            <h2>Subjects</h2>

            <Button
                iconType="left"
                onclick={() => openDialog({ type: "addSubject" })}
            >
                <Icon icon={PlusIcon} />
                Add Subject
            </Button>
        </header>

        <table>
            <thead>
                <tr>
                    <th>Subject / Grade</th>
                    <th>Exams</th>
                    <th>Actions</th>
                </tr>
            </thead>

            <tbody>
                {#each data as row (row.id)}
                    <tr>
                        <td colspan="2">{row.name}</td>
                        <td>
                            <Button
                                onclick={() =>
                                    openDialog({
                                        type: "editSubject",
                                        subjectId: row.id,
                                        name: row.name,
                                    })}
                            >
                                Edit
                            </Button>
                            <Button
                                variant="tonal"
                                color="error"
                                onclick={() => deleteSubject(row.id)}
                            >
                                Delete
                            </Button>
                        </td>
                    </tr>
                    {#each row.subjectGradesSubjectId as grade (grade.grade)}
                        <tr class="indent">
                            <td>{row.name} Grade {grade.grade}</td>
                            <td>
                                <div class="chips">
                                    {#each grade.exams as exam (exam.id)}
                                        <Chip
                                            variant="input"
                                            onclick={() =>
                                                openDialog({
                                                    type: "editExam",
                                                    exam,
                                                })}
                                        >
                                            Paper {exam.paper}
                                        </Chip>
                                    {/each}

                                    <Chip
                                        variant="assist"
                                        icon={PlusIcon}
                                        onclick={() =>
                                            openDialog({
                                                type: "addExam",
                                                subjectId: row.id,
                                                grade: grade.grade,
                                                paper: grade.exams.length + 1,
                                            })}
                                    >
                                        Add Exam
                                    </Chip>
                                </div>
                            </td>
                            <td>
                                <Button
                                    variant="tonal"
                                    color="error"
                                    onclick={() =>
                                        deleteGrade(row.id, grade.grade)}
                                >
                                    Delete
                                </Button>
                            </td>
                        </tr>
                    {/each}
                    <tr class="indent">
                        <td>
                            <Button
                                onclick={() =>
                                    openDialog({
                                        type: "addGrade",
                                        subjectId: row.id,
                                    })}
                                iconType="left"
                                variant="outlined"
                            >
                                <Icon icon={PlusIcon} />
                                Add Grade
                            </Button>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </section>
</main>

<Dialog
    headline="Add Subject"
    bind:open={getDialogOpen("addSubject"), setDialogOpen("addSubject")}
>
    <form
        id="add-subject-form"
        onsubmit={(e) => handleFormSubmit(e, addSubjectForm)}
    >
        <addSubjectForm.Field
            name="name"
            validators={{
                onChange: pipe(string(), nonEmpty("Subject name is required")),
            }}
        >
            {#snippet children(field)}
                <EnhancedTextInput {field} label="Subject Name" required
                ></EnhancedTextInput>
            {/snippet}
        </addSubjectForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant="outlined" onclick={() => (activeDialog = null)}
            >Cancel</Button
        >
        <Button form="add-subject-form" type="submit">Add</Button>
    {/snippet}
</Dialog>

<Dialog
    headline="Edit Subject"
    bind:open={getDialogOpen("editSubject"), setDialogOpen("editSubject")}
>
    <form
        id="edit-subject-form"
        onsubmit={(e) => handleFormSubmit(e, editSubjectForm)}
    >
        <editSubjectForm.Field
            name="name"
            validators={{
                onChange: pipe(string(), nonEmpty("Subject must have a name")),
            }}
        >
            {#snippet children(field)}
                <EnhancedTextInput {field} label="Name" required
                ></EnhancedTextInput>
            {/snippet}
        </editSubjectForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant="outlined" onclick={() => (activeDialog = null)}>
            Cancel
        </Button>
        <Button form="edit-subject-form" type="submit">Edit</Button>
    {/snippet}
</Dialog>

<Dialog
    headline="Add Grade"
    bind:open={getDialogOpen("addGrade"), setDialogOpen("addGrade")}
>
    <form
        id="add-grade-form"
        onsubmit={(e) => handleFormSubmit(e, addGradeForm)}
    >
        <addGradeForm.Field
            name="grade"
            validators={{
                onChange: pipe(
                    number("Grade must be a number"),
                    integer("Grade must be an integer"),
                    minValue(1, "Grade must be at least 1"),
                    maxValue(12, "Grade must be at most 12"),
                ),
            }}
        >
            {#snippet children(field)}
                <EnhancedTextInput
                    {field}
                    label="Grade (e.g. 8 for Grade 8)"
                    type="number"
                    min="1"
                    max="12"
                    required
                ></EnhancedTextInput>
            {/snippet}
        </addGradeForm.Field>
    </form>

    {#snippet buttons()}
        <Button variant="outlined" onclick={() => (activeDialog = null)}>
            Cancel
        </Button>
        <Button form="add-grade-form" type="submit">Add</Button>
    {/snippet}
</Dialog>

<addExamForm.Subscribe
    selector={(state) => ({
        subjectId: state.values.subjectId,
        grade: state.values.grade,
        paper: state.values.paper,
    })}
>
    {#snippet children(state)}
        {@const subjectData = await db.query.subject.findFirst({
            where: { id: state.subjectId },
            columns: { name: true },
        })}

        <Dialog
            headline={`Creating ${subjectData?.name} (Grade ${state.grade}) Paper ${state.paper}`}
            bind:open={getDialogOpen("addExam"), setDialogOpen("addExam")}
        >
            <form
                id="add-exam-form"
                onsubmit={(e) => handleFormSubmit(e, addExamForm)}
            >
                <addExamForm.Field
                    name="durationHours"
                    validators={{
                        onChange: pipe(
                            number("Duration must be a number"),
                            minValue(
                                0.5,
                                "Duration must be at least 0.5 hours",
                            ),
                        ),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label="Duration (hours)"
                            type="number"
                            min="0.5"
                            step="0.25"
                            required
                            helperText="How long the exam lasts for (in hours)"
                        ></EnhancedTextInput>
                    {/snippet}
                </addExamForm.Field>
                <addExamForm.Field
                    name="slotsRequired"
                    validators={{
                        onChange: pipe(
                            number("Slots required must be a number"),
                            integer("Slots required must be an integer"),
                            minValue(1, "Slots required must be at least 1"),
                        ),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label="Slots Required"
                            type="number"
                            min="1"
                            required
                            helperText="How many timeslots this exam runs over. For most exams, this will be 1"
                        ></EnhancedTextInput>
                    {/snippet}
                </addExamForm.Field>

                <addExamForm.Field
                    name="priority"
                    validators={{
                        onChange: pipe(
                            number("Priority must be a number"),
                            integer("Priority must be an integer"),
                            minValue(0, "Priority must be at least 0"),
                        ),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label="Priority"
                            type="number"
                            min="0"
                            step="1"
                            required
                            helperText="Higher priority exams will be scheduled earlier when possible"
                        ></EnhancedTextInput>
                    {/snippet}
                </addExamForm.Field>

                <fieldset>
                    <legend>Timeslot restrictions</legend>
                    <p>
                        Add exact slots this exam is allowed or denied to be
                        scheduled in. If no timeslots are allowed/denied, the
                        exam can be scheduled in any slot. If there are only
                        denied slots, the exam can be scheduled in any slot
                        except the denied ones.
                    </p>
                    <TimeslotPicker
                        {allTimeslots}
                        allowedIds={addExamAllowedIds}
                        deniedIds={addExamDeniedIds}
                    />
                </fieldset>
            </form>

            {#snippet buttons()}
                <Button
                    variant="outlined"
                    onclick={() => (activeDialog = null)}
                >
                    Cancel
                </Button>
                <Button form="add-exam-form" type="submit">Add</Button>
            {/snippet}
        </Dialog>
    {/snippet}
</addExamForm.Subscribe>

<editExamForm.Subscribe
    selector={(state) => ({
        id: state.values.id,
    })}
>
    {#snippet children(state)}
        {@const examData = await db.query.exam.findFirst({
            where: { id: state.id },
            with: { subject: true },
        })}

        <Dialog
            headline="Editing {examData?.subject?.name} Paper {examData?.paper}"
            bind:open={getDialogOpen("editExam"), setDialogOpen("editExam")}
        >
            <form
                id="edit-exam-form"
                onsubmit={(e) => handleFormSubmit(e, editExamForm)}
            >
                <editExamForm.Field
                    name="durationHours"
                    validators={{
                        onChange: pipe(
                            number("Duration must be a number"),
                            minValue(
                                0.5,
                                "Duration must be at least 0.5 hours",
                            ),
                        ),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label="Duration (hours)"
                            type="number"
                            min="0.5"
                            step="0.25"
                            required
                            helperText="How long the exam lasts for (in hours)"
                        ></EnhancedTextInput>
                    {/snippet}
                </editExamForm.Field>
                <editExamForm.Field
                    name="slotsRequired"
                    validators={{
                        onChange: pipe(
                            number("Slots required must be a number"),
                            integer("Slots required must be an integer"),
                            minValue(1, "Slots required must be at least 1"),
                        ),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label="Slots Required"
                            type="number"
                            min="1"
                            required
                            helperText="How many timeslots this exam runs over. For most exams, this will be 1"
                        ></EnhancedTextInput>
                    {/snippet}
                </editExamForm.Field>
                <editExamForm.Field
                    name="priority"
                    validators={{
                        onChange: pipe(
                            number("Priority must be a number"),
                            integer("Priority must be an integer"),
                            minValue(0, "Priority must be at least 0"),
                        ),
                    }}
                >
                    {#snippet children(field)}
                        <EnhancedTextInput
                            {field}
                            label="Priority"
                            type="number"
                            min="0"
                            step="1"
                            required
                            helperText="Higher priority exams will be scheduled earlier when possible"
                        ></EnhancedTextInput>
                    {/snippet}
                </editExamForm.Field>

                <fieldset>
                    <legend>Timeslot restrictions</legend>
                    <p>
                        Add exact slots this exam is allowed or denied to be
                        scheduled in. If no timeslots are allowed/denied, the
                        exam can be scheduled in any slot. If there are only
                        denied slots, the exam can be scheduled in any slot
                        except the denied ones.
                    </p>
                    <TimeslotPicker
                        {allTimeslots}
                        allowedIds={editExamAllowedIds}
                        deniedIds={editExamDeniedIds}
                    />
                </fieldset>
            </form>

            {#snippet buttons()}
                <Button
                    variant="outlined"
                    onclick={() => (activeDialog = null)}
                >
                    Cancel
                </Button>
                <Button
                    variant="tonal"
                    color="error"
                    onclick={() => {
                        editExamForm.handleSubmit({ action: "delete" });
                    }}
                >
                    Delete
                </Button>
                <Button form="edit-exam-form" type="submit">Save</Button>
            {/snippet}
        </Dialog>
    {/snippet}
</editExamForm.Subscribe>

<style>
    h1 {
        @apply --m3-headline-large;

        color: var(--m3c-on-surface);
        padding: 1rem;
        padding-bottom: 0;
    }

    section {
        padding: 1rem;

        header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding-bottom: 0.5rem;
        }
    }

    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    /* <Card variant="outlined"> */
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

    /* Vibe styling */
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
            @apply --m3-label-large

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
                background-color: color-mix(
                    in oklch,
                    var(--m3c-on-surface) 8%,
                    transparent
                );
            }

            /* Subject (parent) rows */
            &:not(.indent) {
                background-color: var(--m3c-surface-container);

                & td:first-child {
                    font-weight: 600;
                    box-shadow: inset 3px 0 0 var(--m3c-primary); /* inset instead of border-left */
                    padding-left: calc(1rem - 3px);
                }
            }

            /* Grade (child) rows */
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
                    }
                }

                /* "Add Grade" button row — only one td */
                &:has(td:only-child) {
                    background-color: transparent;
                    border-bottom: none;

                    & td {
                        padding-block: 0.375rem 0.625rem;
                        padding-left: 2.25rem;
                    }

                    /* Thicker separator before the next subject group */
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
