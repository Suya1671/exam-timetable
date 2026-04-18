<script lang='ts'>
    import type { ICellProps } from '@svar-ui/svelte-grid'
    import type { DeleteStudentEvent, EditStudentEvent, StudentRow } from './types'
    import { Button } from 'm3-svelte'

    const { row, onaction }: ICellProps = $props()

    const student = $derived((row as StudentRow) ?? null)

    /** AI-generated (GPT-5.2-codex). */
    function editStudent() {
        if (!student)
            return

        const data: EditStudentEvent = {
            studentId: student.id,
            studentName: student.name,
            studentGrade: student.grade,
        }

        onaction?.({
            action: 'edit-student',
            data,
        })
    }

    /** AI-generated (GPT-5.2-codex). */
    function deleteStudent() {
        if (!student)
            return

        const data: DeleteStudentEvent = {
            studentId: student.id,
            studentName: student.name,
        }

        onaction?.({
            action: 'delete-student',
            data,
        })
    }
</script>

<div class='row-actions'>
    <Button variant='outlined' onclick={editStudent}>Edit</Button>
    <Button variant='tonal' color='error' onclick={deleteStudent}>Delete</Button>
</div>

<style>
    .row-actions {
        display: flex;
        gap: 0.35rem;
        flex-wrap: nowrap;
        white-space: nowrap;
    }
</style>
