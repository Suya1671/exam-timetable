<script lang="ts">
	import { Button, Dialog } from 'm3-svelte';
	import { createForm } from '@tanstack/svelte-form';
	import EnhancedTextInput from '$lib/EnhancedTextInput.svelte';
	import { pipe, string, nonEmpty } from 'valibot';
	import { createTauRPCProxy, type TimetableDay } from '$lib/backend';

	const backend = createTauRPCProxy();

	let {
		open = $bindable(false),
		onClose,
		title,
		grades,
		days
	}: {
		open?: boolean;
		onClose?: () => void;
		title: string;
		grades: number[];
		days: TimetableDay[];
	} = $props();

	const exportForm = createForm(() => ({
		defaultValues: {
			schoolName: 'School of The Crab',
			title: title || 'Exam Timetable'
		},
		onSubmit: async ({ value }) => {
			const data = {
				schoolName: value.schoolName,
				title: value.title,
				grades,
				days
			};

			try {
				await backend.generate_timetable_pdf(data);
				onClose?.();
			} catch (error) {
				alert(`Failed to generate PDF: ${error}`);
			}
		}
	}));

	/** AI-generated (Gemini). */
	function handleFormSubmit(e: SubmitEvent) {
		e.preventDefault();
		e.stopPropagation();
		exportForm.handleSubmit();
	}
</script>

<Dialog
	headline="Export Timetable PDF"
	bind:open={() => open, (nextOpen: boolean) => !nextOpen && onClose?.()}
>
	<form id="export-timetable-form" onsubmit={handleFormSubmit}>
		<exportForm.Field
			name="schoolName"
			validators={{ onChange: pipe(string(), nonEmpty('School name is required.')) }}
		>
			{#snippet children(field)}
				<EnhancedTextInput {field} label="School name" required placeholder="Enter school name" />
			{/snippet}
		</exportForm.Field>

		<exportForm.Field
			name="title"
			validators={{ onChange: pipe(string(), nonEmpty('Title is required.')) }}
		>
			{#snippet children(field)}
				<EnhancedTextInput
					{field}
					label="Timetable title"
					required
					placeholder="Enter timetable title"
				/>
			{/snippet}
		</exportForm.Field>

		<p class="info-text">
			The exported PDF will include the school name title in the header. The timetable data will be
			embedded into the PDF for future use.
		</p>
	</form>

	{#snippet buttons()}
		<Button variant="outlined" onclick={onClose}>Cancel</Button>
		<Button form="export-timetable-form" type="submit">Export</Button>
	{/snippet}
</Dialog>

<style>
	form {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		min-width: 24rem;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		cursor: pointer;
		user-select: none;
		font-size: 0.95rem;
	}

	.info-text {
		margin: 0;
		opacity: 0.8;
		font-size: 0.875rem;
		line-height: 1.4;
	}
</style>
