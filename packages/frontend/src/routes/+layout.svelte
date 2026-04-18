<script lang='ts'>
    import { page } from '$app/state'
    import IconCalendar from '@ktibow/iconset-material-symbols/calendar-month-rounded'

    import IconGroup from '@ktibow/iconset-material-symbols/group-rounded'
    import IconBook from '@ktibow/iconset-material-symbols/menu-book-rounded'
    import IconSchedule from '@ktibow/iconset-material-symbols/schedule-rounded'
    import IconTune from '@ktibow/iconset-material-symbols/tune-rounded'
    import { NavCMLX, NavCMLXItem } from 'm3-svelte'

    import { onMount } from 'svelte'
    import '../app.css'

    const { children } = $props()

    onMount(async () => {
        if (!('anchorName' in document.documentElement.style)) {
            const { default: polyfill } = await import('@oddbird/css-anchor-positioning/fn')
            await polyfill()
        }
    })

    const items = [
        { href: '/', label: 'Timeslots', icon: IconCalendar },
        { href: '/subjects', label: 'Subjects', icon: IconBook },
        { href: '/constraints', label: 'Constraints', icon: IconTune },
        { href: '/students', label: 'Students', icon: IconGroup },
        { href: '/timetable', label: 'Manage Timetables', icon: IconSchedule },
    ]
</script>

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
