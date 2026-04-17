<!-- TODO: figure out how this works and de-vibe-codify it (On first glance, I can see a lot that can be improved sob) -->
<script lang='ts'>
    import type { timeslot } from '$lib/db/schema'
    import { dateKeyUTC } from '$lib/dateKeys'
    import { Button, Chip, ConnectedButtons, Switch } from 'm3-svelte'
    import { SvelteDate, SvelteMap, SvelteSet } from 'svelte/reactivity'

    type TimeslotRow = typeof timeslot.$inferSelect
    type RestrictionMode = 'allow' | 'deny'

    interface DaySlot {
        id: number
        slot: number
    }

    interface CalendarCell {
        date: Date
        dayNumber: number
        inMonth: boolean
        slots: DaySlot[]
    }

    let {
        allTimeslots,
        selectedIds,
        mode = $bindable<RestrictionMode>(),
    }: {
        allTimeslots: TimeslotRow[]
        selectedIds: SvelteSet<number>
        mode: RestrictionMode
    } = $props()

    const monthName = (month: number) =>
        new Date(2000, month, 1).toLocaleDateString(undefined, {
            month: 'short',
        })

    const slotLabel = (slot: number) => {
        switch (slot) {
            case 0:
                return 'First slot'
            case 1:
                return 'Second slot'
            default:
                return `Slot ${slot + 1}`
        }
    }

    const byDate = $derived.by(() => {
        const map = new SvelteMap<string, DaySlot[]>()

        for (const row of allTimeslots) {
            const slot = typeof row.slot === 'number' ? row.slot : Number(row.slot)

            if (!Number.isFinite(slot)) {
                continue
            }

            const slotInfo = { id: row.id, slot }
            const key = dateKeyUTC(row.date)
            const existing = map.get(key)

            if (existing) {
                existing.push(slotInfo)
            }
            else {
                map.set(key, [slotInfo])
            }
        }

        for (const slots of map.values()) {
            slots.sort((a, b) => a.slot - b.slot)
        }

        return map
    })

    const allDateKeys = $derived([...byDate.keys()].sort())

    const minDateKey = $derived(allDateKeys[0] ?? '')
    const maxDateKey = $derived(allDateKeys[allDateKeys.length - 1] ?? '')

    const minDate = $derived(minDateKey ? new SvelteDate(minDateKey) : null)
    const maxDate = $derived(maxDateKey ? new SvelteDate(maxDateKey) : null)

    const calendarStart = $derived(minDate ? new SvelteDate(minDate) : null)

    let focusedMonth = $derived((calendarStart ?? new SvelteDate()).getMonth())
    let focusedYear = $derived((calendarStart ?? new SvelteDate()).getFullYear())

    const monthIndex = $derived(focusedYear * 12 + focusedMonth)
    const minMonthIndex = $derived(
        minDate ? minDate.getFullYear() * 12 + minDate.getMonth() : monthIndex,
    )
    const maxMonthIndex = $derived(
        maxDate ? maxDate.getFullYear() * 12 + maxDate.getMonth() : monthIndex,
    )

    const canPrevMonth = $derived(monthIndex > minMonthIndex)
    const canNextMonth = $derived(monthIndex < maxMonthIndex)

    const monthTitle = $derived(`${monthName(focusedMonth)} ${focusedYear}`)

    const firstOfFocusedMonth = $derived(new Date(Date.UTC(focusedYear, focusedMonth, 1)))
    const startOffset = $derived(firstOfFocusedMonth.getUTCDay())

    const visibleCells = $derived(
        Array.from({ length: 42 }, (_, index): CalendarCell => {
            const date = new Date(Date.UTC(focusedYear, focusedMonth, index - startOffset + 1))

            return {
                date,
                dayNumber: date.getUTCDate(),
                inMonth: date.getUTCMonth() === focusedMonth,
                slots: byDate.get(dateKeyUTC(date)) ?? [],
            }
        }),
    )

    let showAllCounts = $state(false)
    let dragPointerId = $state<number | null>(null)
    let dragStartX = $state(0)
    let dragStartY = $state(0)
    let dragStartSlotId = $state<number | null>(null)
    let dragStartDayKey = $state<string | null>(null)
    let dragEndDayKey = $state<string | null>(null)
    let dragSetOn = $state<boolean | null>(null)
    let dragActive = $state(false)
    let dragInitialSelected = $state<number[]>([])

    const isSelectedInMode = (timeslotId: number) => selectedIds.has(timeslotId)

    const setSelection = (timeslotId: number, on: boolean) => {
        if (on) {
            selectedIds.add(timeslotId)
        }
        else {
            selectedIds.delete(timeslotId)
        }
    }

    const applySelectionToDay = (slots: DaySlot[], on: boolean) => {
        for (const slot of slots) {
            setSelection(slot.id, on)
        }
    }

    const applySelectionAcrossDayRange = (startDayKey: string, endDayKey: string, on: boolean) => {
        const startIndex = allDateKeys.indexOf(startDayKey)
        const endIndex = allDateKeys.indexOf(endDayKey)
        if (startIndex < 0 || endIndex < 0) {
            return
        }

        const from = Math.min(startIndex, endIndex)
        const to = Math.max(startIndex, endIndex)
        for (let index = from; index <= to; index += 1) {
            const dayKey = allDateKeys[index]
            if (!dayKey) {
                continue
            }
            applySelectionToDay(byDate.get(dayKey) ?? [], on)
        }
    }

    const restoreDragSnapshot = () => {
        selectedIds.clear()
        for (const id of dragInitialSelected) {
            selectedIds.add(id)
        }
    }

    const applyDragRange = (startDayKey: string, endDayKey: string, on: boolean) => {
        restoreDragSnapshot()
        applySelectionAcrossDayRange(startDayKey, endDayKey, on)
    }

    const onSlotPointerDown = (event: PointerEvent, timeslotId: number, dayKey: string) => {
        if (event.button !== 0 && event.button !== 2) {
            return
        }
        event.preventDefault()
        dragPointerId = event.pointerId
        dragStartX = event.clientX
        dragStartY = event.clientY
        dragStartSlotId = timeslotId
        dragStartDayKey = dayKey
        dragEndDayKey = dayKey
        dragSetOn = event.button === 2 ? false : !isSelectedInMode(timeslotId)
        dragActive = false
        dragInitialSelected = Array.from(selectedIds)
    }

    /** AI-generated (GPT-5.2-codex). */
    const onGridContextMenu = (event: MouseEvent) => {
        event.preventDefault()
    }

    /** AI-generated (GPT-5.2-codex). */
    const toggleDayInMode = (slots: DaySlot[]) => {
        const allSelected = slots.every(slot => isSelectedInMode(slot.id))
        applySelectionToDay(slots, !allSelected)
    }

    /** AI-generated (GPT-5.2-codex). */
    const onWindowPointerMove = (event: PointerEvent) => {
        if (dragPointerId !== event.pointerId || dragSetOn === null) {
            return
        }

        const movedEnough
            = Math.abs(event.clientX - dragStartX) > 3 || Math.abs(event.clientY - dragStartY) > 3

        if (!dragActive && movedEnough) {
            dragActive = true
            if (dragStartDayKey) {
                applyDragRange(dragStartDayKey, dragStartDayKey, dragSetOn)
            }
        }

        if (!dragActive) {
            return
        }

        const target = document.elementFromPoint(
            event.clientX,
            event.clientY,
        ) as HTMLElement | null
        const container = target?.closest<HTMLElement>('[data-day-key]')
        if (!container) {
            return
        }
        const dayKey = container.dataset.dayKey
        if (!dayKey || !dragStartDayKey || dayKey === dragEndDayKey) {
            return
        }
        dragEndDayKey = dayKey
        applyDragRange(dragStartDayKey, dayKey, dragSetOn)
    }

    /** AI-generated (GPT-5.2-codex). */
    const onWindowPointerUp = (event: PointerEvent) => {
        if (dragPointerId !== event.pointerId) {
            return
        }

        if (!dragActive && dragStartSlotId !== null && dragSetOn !== null) {
            setSelection(dragStartSlotId, dragSetOn)
        }

        dragPointerId = null
        dragStartSlotId = null
        dragStartDayKey = null
        dragEndDayKey = null
        dragSetOn = null
        dragActive = false
        dragInitialSelected = []
    }

    const onSlotKey = (event: KeyboardEvent, timeslotId: number) => {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault()
            setSelection(timeslotId, !isSelectedInMode(timeslotId))
            return
        }

        if (event.key.toLowerCase() === 'd') {
            event.preventDefault()
            mode = 'deny'
            setSelection(timeslotId, !isSelectedInMode(timeslotId))
            return
        }

        if (event.key.toLowerCase() === 'a') {
            event.preventDefault()
            mode = 'allow'
            setSelection(timeslotId, !isSelectedInMode(timeslotId))
            return
        }

        if (event.key === 'Delete' || event.key === 'Backspace') {
            event.preventDefault()
            setSelection(timeslotId, false)
        }
    }

    const slotState = (timeslotId: number) => ({
        selected: selectedIds.has(timeslotId),
    })

    const clearCurrentMode = () => {
        selectedIds.clear()
    }

    const moveMonth = (delta: number) => {
        const nextIndex = monthIndex + delta
        if (nextIndex < minMonthIndex || nextIndex > maxMonthIndex) {
            return
        }

        const nextYear = Math.floor(nextIndex / 12)
        const nextMonth = nextIndex % 12
        focusedYear = nextYear
        focusedMonth = nextMonth
    }

    const selectedCount = $derived(selectedIds.size)
    const visibleTimeslotIds = $derived.by(() => {
        const ids = new SvelteSet<number>()
        for (const cell of visibleCells) {
            if (!cell.inMonth) {
                continue
            }
            for (const slot of cell.slots) {
                ids.add(slot.id)
            }
        }
        return ids
    })
    const visibleTotalCount = $derived(visibleTimeslotIds.size)
    const visibleSelectedCount = $derived.by(() => {
        let count = 0
        for (const id of visibleTimeslotIds) {
            if (selectedIds.has(id)) {
                count += 1
            }
        }
        return count
    })
    const visibleUnsetCount = $derived.by(() => visibleTotalCount - visibleSelectedCount)
    const selectedCountDisplay = $derived(showAllCounts ? selectedCount : visibleSelectedCount)
    const unsetCountDisplay = $derived(
        showAllCounts ? Math.max(0, allTimeslots.length - selectedCount) : visibleUnsetCount,
    )
    const minDateLabel = $derived(minDate?.toLocaleDateString() ?? '')
    const maxDateLabel = $derived(maxDate?.toLocaleDateString() ?? '')

    const slotOneTimeslots = $derived(allTimeslots.filter(row => Number(row.slot) === 0))
    const slotTwoTimeslots = $derived(allTimeslots.filter(row => Number(row.slot) === 1))

    /** AI-generated (GPT-5.2-codex). */
    const batchApply = (scope: 'all' | 'slot1' | 'slot2') => {
        const targets
            = scope === 'all'
                ? allTimeslots
                : scope === 'slot1'
                ? slotOneTimeslots
                : slotTwoTimeslots
        for (const slot of targets) {
            setSelection(slot.id, true)
        }
    }

    const topSlotFor = (slots: DaySlot[]) => slots[0]
    const bottomSlotFor = (slots: DaySlot[]) => slots[1]
</script>

<section
    class='timeslot-picker'
    class:mode-allow={mode === 'allow'}
    class:mode-deny={mode === 'deny'}
>
    <header class='calendar-header'>
        <button
            type='button'
            class='arrow m3-layer'
            onclick={() => moveMonth(-1)}
            disabled={!canPrevMonth}
            aria-label='Previous month'
        >
            &lsaquo;
        </button>
        <h4>{monthTitle}</h4>
        <button
            type='button'
            class='arrow m3-layer'
            onclick={() => moveMonth(1)}
            disabled={!canNextMonth}
            aria-label='Next month'
        >
            &rsaquo;
        </button>
    </header>

    <div class='info-section'>
        <div class='toolbar-row'>
            <ConnectedButtons>
                <Button variant='filled' label square={true}>
                    <input type='radio' name='restriction-mode' value='allow' bind:group={mode} />
                    Allow
                </Button>
                <Button variant='filled' label square={true}>
                    <input type='radio' name='restriction-mode' value='deny' bind:group={mode} />
                    Deny
                </Button>
            </ConnectedButtons>
            <Chip variant='assist' onclick={clearCurrentMode}>
                Clear {mode}
            </Chip>
            <label class='count-toggle'>
                <Switch bind:checked={showAllCounts} />
                All months
            </label>
        </div>
        <div class='info-row'>
            <span class='pill summary' class:allow={mode === 'allow'} class:deny={mode === 'deny'}>
                {mode === 'allow' ? 'Allowed' : 'Denied'}
                {selectedCountDisplay}
            </span>
            <span class='pill neutral'>Unset {unsetCountDisplay}</span>
        </div>
        <p class='hint'>
            Left click toggles one slot. Right click clears. Drag paints a date range. Click a day
            number to toggle the whole day.
        </p>
    </div>

    <hr />

    <div class='batch-grid'>
        <div class='batch-section'>
            <div class='section-title'>Batch actions</div>
            <div class='chip-row'>
                <Chip variant='assist' onclick={() => batchApply('all')}>
                    {mode === 'allow' ? 'Allow all' : 'Deny all'}
                </Chip>
                <Chip variant='assist' onclick={() => batchApply('slot1')}>
                    {mode === 'allow' ? 'Allow slot 1' : 'Deny slot 1'}
                </Chip>
                <Chip variant='assist' onclick={() => batchApply('slot2')}>
                    {mode === 'allow' ? 'Allow slot 2' : 'Deny slot 2'}
                </Chip>
                <Chip variant='assist' onclick={clearCurrentMode}>Clear all</Chip>
            </div>
        </div>
    </div>

    <p class='hint'>
        Allowed list has priority over denied list. Switching mode clears the other list.
    </p>

    <div class='weekday-row'>
        {#each ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as day}
            <div class='weekday'>{day}</div>
        {/each}
    </div>

    <div class='calendar-grid'>
        {#each visibleCells as cell (cell.date.getTime())}
            <article
                class='day-cell'
                class:outside={!cell.inMonth}
                data-day-key={dateKeyUTC(cell.date)}
            >
                {#if cell.inMonth && cell.slots.length > 0}
                    <button
                        type='button'
                        class='day-number button-reset'
                        onclick={() => toggleDayInMode(cell.slots)}
                        oncontextmenu={(event) => {
                            onGridContextMenu(event)
                            applySelectionToDay(cell.slots, false)
                        }}
                    >
                        {cell.dayNumber}
                    </button>
                {:else}
                    <div class='day-number'>{cell.dayNumber}</div>
                {/if}

                {#if cell.inMonth && cell.slots.length > 0}
                    {@const top = topSlotFor(cell.slots)}
                    {@const bottom = bottomSlotFor(cell.slots)}

                    <div class='split-circle' role='group' aria-label='Timeslots for date'>
                        {#if top}
                            {@const state = slotState(top.id)}
                            <button
                                type='button'
                                class='half top'
                                class:selected={state.selected}
                                data-timeslot-id={top.id}
                                title={slotLabel(top.slot)}
                                oncontextmenu={onGridContextMenu}
                                onpointerdown={event =>
                                    onSlotPointerDown(event, top.id, dateKeyUTC(cell.date))}
                                onkeydown={event => onSlotKey(event, top.id)}
                            >
                                1
                            </button>
                        {:else}
                            <div class='half top placeholder'></div>
                        {/if}

                        {#if bottom}
                            {@const state = slotState(bottom.id)}
                            <button
                                type='button'
                                class='half bottom'
                                class:selected={state.selected}
                                data-timeslot-id={bottom.id}
                                title={slotLabel(bottom.slot)}
                                oncontextmenu={onGridContextMenu}
                                onpointerdown={event =>
                                    onSlotPointerDown(event, bottom.id, dateKeyUTC(cell.date))}
                                onkeydown={event => onSlotKey(event, bottom.id)}
                            >
                                2
                            </button>
                        {:else}
                            <div class='half bottom placeholder'></div>
                        {/if}
                    </div>
                {/if}
            </article>
        {/each}
    </div>

    {#if minDate && maxDate}
        <p class='bounds'>Timetable range: {minDateLabel} to {maxDateLabel}</p>
    {/if}
</section>

<svelte:window
    onpointerup={onWindowPointerUp}
    onpointercancel={onWindowPointerUp}
    onpointermove={onWindowPointerMove}
/>

<style>
    .timeslot-picker {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        --selection-bg: var(--m3c-primary);
        --selection-fg: var(--m3c-on-primary);
        --summary-bg: var(--m3c-primary-container);
        --summary-fg: var(--m3c-on-primary-container);
    }

    .timeslot-picker.mode-deny {
        --selection-bg: var(--m3c-error);
        --selection-fg: var(--m3c-on-error);
        --summary-bg: color-mix(in oklch, var(--m3c-error) 22%, white);
        --summary-fg: var(--m3c-error);
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
        color: color-mix(in oklch, var(--m3c-on-surface-variant) 38%, transparent);
        cursor: default;
    }

    .info-section {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
        padding-inline: 0.15rem;
    }

    .toolbar-row,
    .info-row {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.4rem;
    }

    .chip-row {
        display: inline-flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.4rem;
    }

    .batch-section {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
        padding-inline: 0.15rem;
    }

    .batch-grid {
        display: grid;
        gap: 0.6rem;
        padding-inline: 0.15rem;
    }

    .section-title {
        @apply --m3-label-large;
        color: var(--m3c-on-surface);
    }

    .pill {
        @apply --m3-label;
        border-radius: 999px;
        padding: 0rem 1rem;
        height: 2rem;
        display: inline-flex;
        align-items: center;
    }

    .pill.summary {
        background: var(--summary-bg);
        color: var(--summary-fg);
    }

    .pill.neutral {
        background: var(--m3c-surface-container-highest);
        color: var(--m3c-on-surface-variant);
    }

    .count-toggle {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        @apply --m3-body-small;
        color: var(--m3c-on-surface-variant);
        padding-inline: 0.2rem;
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
        background: transparent;
    }

    .button-reset {
        border: none;
        padding: 0;
        margin: 0;
        cursor: pointer;
    }

    .button-reset:hover {
        color: var(--m3c-primary);
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

    .half.selected {
        background: var(--selection-bg);
        color: var(--selection-fg);
    }

    .half:focus-visible {
        outline: 2px solid var(--m3c-primary);
        outline-offset: 1px;
    }

    .half:not(.selected):not(.placeholder):hover {
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

    hr {
        border: none;
        border-top: 1px solid var(--m3c-outline-variant);
        margin: 1rem 0;
    }
</style>
