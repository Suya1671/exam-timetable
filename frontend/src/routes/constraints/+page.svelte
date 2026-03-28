<script lang="ts">
	import './styles.css';
	import SameDayConstraintsCard from './SameDayConstraintsCard.svelte';
	import SameTimeConstraintsCard from './SameTimeConstraintsCard.svelte';
	import DifferentWeekConstraintsCard from './DifferentWeekConstraintsCard.svelte';

	let sameTimeCount = $state(0);
	let sameDayCount = $state(0);
	let differentWeekCount = $state(0);

	/** AI-generated (GPT-5.3-codex). */
	function handleSameTimeCountChange(count: number) {
		sameTimeCount = count;
	}

	/** AI-generated (GPT-5.3-codex). */
	function handleSameDayCountChange(count: number) {
		sameDayCount = count;
	}

	/** AI-generated (GPT-5.3-codex). */
	function handleDifferentWeekCountChange(count: number) {
		differentWeekCount = count;
	}
</script>

<header data-route="constraints">
	<div>
		<h1>Constraints</h1>
		<h2>Setup additional constraints for the timetable</h2>
	</div>
	<ul aria-label="Constraint counts">
		<li>Same-time: {sameTimeCount}</li>
		<li>Same-day: {sameDayCount}</li>
		<li>Different-week: {differentWeekCount}</li>
	</ul>
</header>

<main data-route="constraints">
	<SameTimeConstraintsCard onCountChange={handleSameTimeCountChange} />
	<SameDayConstraintsCard onCountChange={handleSameDayCountChange} />
	<DifferentWeekConstraintsCard onCountChange={handleDifferentWeekCountChange} />
</main>
