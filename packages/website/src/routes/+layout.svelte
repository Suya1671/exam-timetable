<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';

    import '../app.css'
    import '@exam-timetable/ui/theme.css'
    import '@exam-timetable/ui/reset.css'

    const { children } = $props()
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
