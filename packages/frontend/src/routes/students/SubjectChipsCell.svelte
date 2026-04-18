<script lang='ts'>
    import type { ICellProps } from '@svar-ui/svelte-grid'
    import type { StudentRow, Subject } from './types'
    import PlusIcon from '@ktibow/iconset-material-symbols/add-rounded'
    import RemoveIcon from '@ktibow/iconset-material-symbols/close-rounded'
    import PlaylistAddIcon from '@ktibow/iconset-material-symbols/playlist-add-rounded'
    import { Chip } from 'm3-svelte'

    const { row, onaction }: ICellProps = $props()

    const student = $derived((row as StudentRow) ?? null)
    const subjects = $derived((student?.subjects ?? []) as Subject[])

    function removeSubject(subject: Subject) {
        onaction?.({
            action: 'remove-subject',
            data: {
                studentId: student?.id,
                subjectId: subject.id,
                subjectName: subject.name,
            },
        })
    }
</script>

<div class='chips' role='group' aria-label='Student subjects'>
    {#each subjects as subject (subject.id)}
        <Chip
            variant='input'
            onclick={() => removeSubject(subject)}
            title={`Remove ${subject.name}`}
            icon={RemoveIcon}
        >
            {subject.name}
        </Chip>
    {/each}

    <Chip
        variant='assist'
        icon={PlusIcon}
        onclick={() => onaction?.({ action: 'add-subject', data: { studentId: student?.id } })}
        title='Add subject'
    >
        Add subject
    </Chip>

    <Chip
        variant='assist'
        icon={PlaylistAddIcon}
        onclick={() => onaction?.({ action: 'add-all-subjects', data: { studentId: student?.id } })}
    >
        Add all subjects
    </Chip>

    <Chip
        variant='assist'
        icon={RemoveIcon}
        onclick={() =>
            onaction?.({
                action: 'clear-subjects',
                data: { studentId: student?.id, studentName: student?.name },
            })}
        color='error'
        title='Clear subjects'
    >
        Clear subjects
    </Chip>
</div>

<style>
    .chips {
        display: flex;
        flex-wrap: wrap;
        gap: 0.35rem;
        align-items: center;
        align-content: center;
        width: 100%;
        min-height: 2.5rem;
        padding-block: 0.2rem;
        box-sizing: border-box;
        contain: layout inline-size;
    }
</style>
