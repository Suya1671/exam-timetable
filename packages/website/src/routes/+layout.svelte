<script lang='ts'>
    import { page } from '$app/state'
    import favicon from '$lib/assets/favicon.svg'

    import IconCalendar from '@ktibow/iconset-material-symbols/calendar-month-rounded'

    import IconTeacher from '@ktibow/iconset-material-symbols/school-rounded'
    import { NavCMLX, NavCMLXItem } from 'm3-svelte'

    import '../app.css'
    import '@exam-timetable/ui/theme.css'
    import '@exam-timetable/ui/reset.css'

    const { children } = $props()

    const items = [
        { href: '/', label: 'Student Timetable', icon: IconCalendar },
        { href: '/teachers', label: 'Teacher Timetable', icon: IconTeacher },
    ]
</script>

<svelte:head>
    <link rel='icon' href={favicon} />
</svelte:head>

<aside>
    <NavCMLX variant='auto'>
        {#each items as item (item.href)}
            {@const path = page.url.pathname}
            {@const selected = item.href === '/' ? item.href === path : path.startsWith(item.href)}

            <NavCMLXItem
                variant='auto'
                text={item.label}
                href={item.href}
                icon={item.icon}
                {selected}
            />
        {/each}
    </NavCMLX>
</aside>

{@render children()}

<style>
    aside {
        grid-area: sidebar;
        align-self: stretch;
        block-size: 100%;
        display: grid;
        place-items: center;

        @media (width < 52.5rem) {
            :global(nav) {
                width: 100%;
            }
        }
    }
</style>
