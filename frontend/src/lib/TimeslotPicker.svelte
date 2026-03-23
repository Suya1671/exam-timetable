<!-- TODO: figure out how this works and de-vibe-codify it (On first glance, I can see a lot that can be improved sob) -->
<!-- So far: replaced maps with sveltemaps and sets with sveltesets, gotta try lower the amount of random cache variables n shit -->
<script lang="ts">
    import { SvelteDate, SvelteSet, SvelteMap } from "svelte/reactivity";
    import type { timeslot } from "$lib/db/schema";
    type TimeslotRow = typeof timeslot.$inferSelect;

    interface DaySlot {
        id: number;
        slot: number;
    }

    interface CalendarCell {
        date: Date;
        dayNumber: number;
        inMonth: boolean;
        slots: DaySlot[];
    }

    let {
        allTimeslots,
        allowedIds,
        deniedIds,
    }: {
        allTimeslots: TimeslotRow[];
        allowedIds: SvelteSet<number>;
        deniedIds: SvelteSet<number>;
    } = $props();

    const monthName = (month: number) =>
        new Date(2000, month, 1).toLocaleDateString(undefined, {
            month: "short",
        });

    const slotLabel = (slot: number) => {
        switch (slot) {
            case 0:
                return "First slot";
            case 1:
                return "Second slot";
            default:
                return `Slot ${slot + 1}`;
        }
    };

    const byDate = $derived.by(() => {
        const map = new SvelteMap<Date, DaySlot[]>();

        for (const row of allTimeslots) {
            const slot =
                typeof row.slot === "number" ? row.slot : Number(row.slot);

            if (!Number.isFinite(slot)) {
                continue;
            }

            const slotInfo = { id: row.id, slot };
            const existing = map.get(row.date);

            if (existing) {
                existing.push(slotInfo);
            } else {
                map.set(row.date, [slotInfo]);
            }
        }

        for (const slots of map.values()) {
            slots.sort((a, b) => a.slot - b.slot);
        }

        return map;
    });

    const allDateKeys = $derived(
        [...byDate.keys()].sort((a, b) => a.getTime() - b.getTime()),
    );

    const minDateKey = $derived(allDateKeys[0] ?? "");
    const maxDateKey = $derived(allDateKeys[allDateKeys.length - 1] ?? "");

    const minDate = $derived(
        minDateKey ? new SvelteDate(`${minDateKey}T00:00:00`) : null,
    );
    const maxDate = $derived(
        maxDateKey ? new SvelteDate(`${maxDateKey}T00:00:00`) : null,
    );

    const calendarStart = $derived(
        minDate
            ? new SvelteDate(minDate.getFullYear(), minDate.getMonth(), 1)
            : null,
    );

    let focusedMonth = $derived((calendarStart ?? new SvelteDate()).getMonth());
    let focusedYear = $derived(
        (calendarStart ?? new SvelteDate()).getFullYear(),
    );

    const monthIndex = $derived(focusedYear * 12 + focusedMonth);
    const minMonthIndex = $derived(
        minDate ? minDate.getFullYear() * 12 + minDate.getMonth() : monthIndex,
    );
    const maxMonthIndex = $derived(
        maxDate ? maxDate.getFullYear() * 12 + maxDate.getMonth() : monthIndex,
    );

    const canPrevMonth = $derived(monthIndex > minMonthIndex);
    const canNextMonth = $derived(monthIndex < maxMonthIndex);

    const monthTitle = $derived(`${monthName(focusedMonth)} ${focusedYear}`);

    const visibleCells = $derived.by(() => {
        const first = new Date(focusedYear, focusedMonth, 1);
        const startOffset = first.getDay();

        return Array.from({ length: 42 }, (_, index): CalendarCell => {
            const date = new Date(
                focusedYear,
                focusedMonth,
                index - startOffset + 1,
            );
            return {
                date,
                dayNumber: date.getDate(),
                inMonth: date.getMonth() === focusedMonth,
                slots: byDate.get(date) ?? [],
            };
        });
    });

    const setAllowed = (timeslotId: number, on: boolean) => {
        if (on) {
            allowedIds.add(timeslotId);
            deniedIds.delete(timeslotId);
        } else {
            allowedIds.delete(timeslotId);
        }
    };

    const setDenied = (timeslotId: number, on: boolean) => {
        if (on) {
            deniedIds.add(timeslotId);
            allowedIds.delete(timeslotId);
        } else {
            deniedIds.delete(timeslotId);
        }
    };

    const onSlotClick = (timeslotId: number) => {
        const isAllowed = allowedIds.has(timeslotId);
        setAllowed(timeslotId, !isAllowed);
    };

    const onSlotContext = (event: MouseEvent, timeslotId: number) => {
        event.preventDefault();
        const isDenied = deniedIds.has(timeslotId);
        setDenied(timeslotId, !isDenied);
    };

    const onSlotKey = (event: KeyboardEvent, timeslotId: number) => {
        if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            const isAllowed = allowedIds.has(timeslotId);
            setAllowed(timeslotId, !isAllowed);
            return;
        }

        if (event.key.toLowerCase() === "d") {
            event.preventDefault();
            const isDenied = deniedIds.has(timeslotId);
            setDenied(timeslotId, !isDenied);
            return;
        }

        if (event.key.toLowerCase() === "a") {
            event.preventDefault();
            const isAllowed = allowedIds.has(timeslotId);
            setAllowed(timeslotId, !isAllowed);
            return;
        }

        if (event.key === "Delete" || event.key === "Backspace") {
            event.preventDefault();
            setAllowed(timeslotId, false);
            setDenied(timeslotId, false);
        }
    };

    const slotState = (timeslotId: number) => ({
        allowed: allowedIds.has(timeslotId),
        denied: deniedIds.has(timeslotId),
    });

    const clearAllowed = () => {
        allowedIds.clear();
    };

    const clearDenied = () => {
        deniedIds.clear();
    };

    const moveMonth = (delta: number) => {
        const nextIndex = monthIndex + delta;
        if (nextIndex < minMonthIndex || nextIndex > maxMonthIndex) {
            return;
        }

        const nextYear = Math.floor(nextIndex / 12);
        const nextMonth = nextIndex % 12;
        focusedYear = nextYear;
        focusedMonth = nextMonth;
    };

    const allowedCount = $derived(allowedIds.size);
    const deniedCount = $derived(deniedIds.size);
    const minDateLabel = $derived(minDate?.toLocaleDateString() ?? "");
    const maxDateLabel = $derived(maxDate?.toLocaleDateString() ?? "");

    const topSlotFor = (slots: DaySlot[]) => slots[0];
    const bottomSlotFor = (slots: DaySlot[]) => slots[1];
</script>

<section class="timeslot-picker">
    <header class="calendar-header">
        <button
            type="button"
            class="arrow m3-layer"
            onclick={() => moveMonth(-1)}
            disabled={!canPrevMonth}
            aria-label="Previous month"
        >
            &lsaquo;
        </button>
        <h4>{monthTitle}</h4>
        <button
            type="button"
            class="arrow m3-layer"
            onclick={() => moveMonth(1)}
            disabled={!canNextMonth}
            aria-label="Next month"
        >
            &rsaquo;
        </button>
    </header>

    <div class="status-row">
        <span class="pill allowed">Allowed {allowedCount}</span>
        <span class="pill denied">Denied {deniedCount}</span>
        <button type="button" class="small" onclick={clearAllowed}
            >Clear allowed</button
        >
        <button type="button" class="small" onclick={clearDenied}
            >Clear denied</button
        >
    </div>

    <p class="hint">
        Left click allows. Right click denies. (Keyboard: Enter/Space allow, D
        deny, Delete clear.)
    </p>

    <div class="weekday-row">
        {#each ["S", "M", "T", "W", "T", "F", "S"] as day}
            <div class="weekday">{day}</div>
        {/each}
    </div>

    <div class="calendar-grid">
        {#each visibleCells as cell ((cell.date.getTime(), cell.inMonth))}
            <article class="day-cell" class:outside={!cell.inMonth}>
                <div class="day-number">{cell.dayNumber}</div>

                {#if cell.inMonth && cell.slots.length > 0}
                    {@const top = topSlotFor(cell.slots)}
                    {@const bottom = bottomSlotFor(cell.slots)}

                    <div
                        class="split-circle"
                        role="group"
                        aria-label="Timeslots for date"
                    >
                        {#if top}
                            {@const state = slotState(top.id)}
                            <button
                                type="button"
                                class="half top"
                                class:allowed={state.allowed}
                                class:denied={state.denied}
                                title={slotLabel(top.slot)}
                                onclick={() => onSlotClick(top.id)}
                                oncontextmenu={(event) =>
                                    onSlotContext(event, top.id)}
                                onkeydown={(event) => onSlotKey(event, top.id)}
                            >
                                1
                            </button>
                        {:else}
                            <div class="half top placeholder"></div>
                        {/if}

                        {#if bottom}
                            {@const state = slotState(bottom.id)}
                            <button
                                type="button"
                                class="half bottom"
                                class:allowed={state.allowed}
                                class:denied={state.denied}
                                title={slotLabel(bottom.slot)}
                                onclick={() => onSlotClick(bottom.id)}
                                oncontextmenu={(event) =>
                                    onSlotContext(event, bottom.id)}
                                onkeydown={(event) =>
                                    onSlotKey(event, bottom.id)}
                            >
                                2
                            </button>
                        {:else}
                            <div class="half bottom placeholder"></div>
                        {/if}
                    </div>
                {/if}
            </article>
        {/each}
    </div>

    {#if minDateKey && maxDateKey}
        <p class="bounds">Timetable range: {minDateLabel} to {maxDateLabel}</p>
    {/if}
</section>

<style>
    .timeslot-picker {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .calendar-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        height: 2.5rem;
        padding-inline: 0.15rem;
    }

    .calendar-header h4 {
        @apply --m3-title-medium;
        margin: 0;
        color: var(--m3c-on-surface);
    }

    .arrow {
        width: 2rem;
        height: 2rem;
        border-radius: 999px;
        border: none;
        background: transparent;
        color: var(--m3c-on-surface-variant);
        cursor: pointer;
        font-size: 1.25rem;
        line-height: 1;
    }

    .arrow:not(:disabled):hover {
        background: color-mix(in oklch, var(--m3c-on-surface) 8%, transparent);
    }

    .arrow:disabled {
        color: color-mix(
            in oklch,
            var(--m3c-on-surface-variant) 38%,
            transparent
        );
        cursor: default;
    }

    .status-row {
        display: inline-flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.3rem;
        padding-inline: 0.15rem;
    }

    .pill {
        @apply --m3-label-small;
        border-radius: 999px;
        padding: 0.16rem 0.45rem;
    }

    .pill.allowed {
        background: var(--m3c-primary-container);
        color: var(--m3c-on-primary-container);
    }

    .pill.denied {
        background: color-mix(in oklch, var(--m3c-error) 22%, white);
        color: var(--m3c-error);
    }

    .small {
        @apply --m3-label-small;
        height: 1.5rem;
        border-radius: 999px;
        border: none;
        background: transparent;
        color: var(--m3c-on-surface);
        padding: 0 0.45rem;
        cursor: pointer;
    }

    .small:hover {
        background: color-mix(in oklch, var(--m3c-on-surface) 8%, transparent);
    }

    .hint,
    .bounds {
        @apply --m3-body-small;
        margin: 0;
        color: var(--m3c-on-surface-variant);
        padding-inline: 0.15rem;
    }

    .weekday-row {
        display: grid;
        grid-template-columns: repeat(7, 1fr);
        gap: 0.2rem;
        padding-inline: 0.15rem;
    }

    .weekday {
        @apply --m3-body-small;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        height: 1.85rem;
        color: var(--m3c-on-surface-variant);
    }

    .calendar-grid {
        display: grid;
        grid-template-columns: repeat(7, 1fr);
        gap: 0.2rem;
        max-height: 16.4rem;
        overflow: auto;
        padding-inline: 0.15rem;
        padding-right: 0.2rem;
    }

    .day-cell {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.24rem;
        min-height: 4.25rem;
        border-radius: var(--m3-shape-small);
        padding: 0.15rem 0.05rem;
        background: transparent;
    }

    .day-number {
        @apply --m3-label-medium;
        height: 1rem;
        color: var(--m3c-on-surface);
        opacity: 0.94;
    }

    .day-cell.outside .day-number {
        color: color-mix(in oklch, var(--m3c-on-surface) 45%, transparent);
    }

    .split-circle {
        width: 2.15rem;
        height: 2.15rem;
        border-radius: 50%;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        border: 1px solid var(--m3c-outline-variant);
        background: var(--m3c-surface-container-highest);
        box-shadow: var(--m3-elevation-1);
    }

    .half {
        @apply --m3-label-small;
        width: 100%;
        border: none;
        color: var(--m3c-on-surface-variant);
        padding: 0;
        height: 50%;
        line-height: 1;
        cursor: pointer;
        user-select: none;
        background: transparent;
        transition: background-color 120ms ease;
    }

    .half.top {
        border-bottom: 1px solid var(--m3c-outline-variant);
    }

    .half.placeholder {
        cursor: default;
        opacity: 0.36;
    }

    .half.allowed {
        background: var(--m3c-primary);
        color: var(--m3c-on-primary);
    }

    .half.denied {
        background: var(--m3c-error);
        color: var(--m3c-on-error);
    }

    .half:focus-visible {
        outline: 2px solid var(--m3c-primary);
        outline-offset: 1px;
    }

    .half:not(.allowed):not(.denied):not(.placeholder):hover {
        background: color-mix(in oklch, var(--m3c-primary) 16%, transparent);
        color: var(--m3c-primary);
    }

    @media (max-width: 40rem) {
        .calendar-grid {
            max-height: 14.4rem;
        }

        .day-cell {
            min-height: 4.2rem;
        }

        .split-circle {
            width: 1.95rem;
            height: 1.95rem;
        }

        .half {
            font-size: 0.62rem;
        }
    }
</style>
