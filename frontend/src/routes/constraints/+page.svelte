<script lang="ts">
	import TimeConstraints from './TimeConstraints.svelte';
	import OrderConstraints from './OrderConstraints.svelte';
</script>

<header>
	<h1>Constraints</h1>
	<h2>Setup additional constraints for the timetable</h2>
</header>

<main>
	<TimeConstraints />
	<OrderConstraints />
</main>

<style>
	header {
		margin-bottom: 1.5rem;

		h1 {
			@apply --m3-headline-medium;
			color: var(--m3c-on-surface);
		}

		h2 {
			@apply --m3-body-large;
			color: var(--m3c-on-surface-variant);
		}
	}

	main {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
