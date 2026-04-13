<script lang="ts">
	import type { IconifyIcon } from '@iconify/types';
	import type { HTMLInputAttributes } from 'svelte/elements';

	import { TextField } from 'm3-svelte';
	import { useStore, type AnyFieldApi } from '@tanstack/svelte-form';

	interface Props extends HTMLInputAttributes {
		label: string;
		field: AnyFieldApi;
		disabled?: boolean;
		required?: boolean;
		helperText?: string | null;
		leadingIcon?: IconifyIcon;
	}

	let {
		field,
		required = false,
		label,
		helperText = null,
		type,
		...extraOptions
	}: Props = $props();

	const isNumeric = $derived(type === 'number' || type === 'range' || type === 'tel');

	const { current: isSubmitting } = $derived(
		useStore(field.form.store, (state) => state.isSubmitting)
	);
	const disabled = $derived(isSubmitting);

	const { current: errors } = $derived(useStore(field.store, (state) => state.meta.errors));
</script>

<div style="display: flex;flex-direction: column;gap: 0.25rem;">
	<TextField
		name={field.name}
		bind:value={field.state.value}
		oninput={(e) =>
			field.handleChange(isNumeric ? e.currentTarget.valueAsNumber : e.currentTarget.value)}
		onblur={() => field.handleBlur()}
		error={errors.length !== 0}
		{type}
		{label}
		{disabled}
		{required}
		{...extraOptions}
	/>
	<div
		class="m3-font-body-small"
		style="padding: 0px 1rem;color: rgb(var(--m3-scheme-on-surface-variant));"
	>
		{#if errors.length > 0}
			<ul class="ma-auto pl-0 text-red">
				<!-- eslint-disable-next-line svelte/require-each-key -->
				{#each errors as error}
					<li>
						{#if typeof error === 'string'}
							{error}
						{:else}
							{error?.message ?? JSON.stringify(error)}
						{/if}
					</li>
				{/each}
			</ul>
		{:else if helperText !== null}
			{helperText}
		{/if}
	</div>
</div>
