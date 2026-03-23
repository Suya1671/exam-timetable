<script lang="ts">
    import "../app.css";
    import { page } from "$app/state";

    import IconHome from "@ktibow/iconset-material-symbols/home-rounded";
    import IconGroup from "@ktibow/iconset-material-symbols/group-rounded";
    import IconTune from "@ktibow/iconset-material-symbols/tune-rounded";
    import IconSchedule from "@ktibow/iconset-material-symbols/schedule-rounded";

    import { NavCMLX, NavCMLXItem } from "m3-svelte";

    let { children } = $props();

    const items = [
        { href: "/", label: "Information", icon: IconHome },
        { href: "/constraints", label: "Constraints", icon: IconTune },
        { href: "/students", label: "Students", icon: IconGroup },
        { href: "/timetable", label: "Create Timetable", icon: IconSchedule },
    ];
</script>

<aside>
    <NavCMLX variant="auto">
        {#each items as item}
            {@const path = page.url.pathname}
            {@const selected =
                item.href === "/"
                    ? item.href === path
                    : path.startsWith(item.href)}

            <NavCMLXItem
                variant="auto"
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

    :global(> header) {
        grid-area: header;
    }

    :global(> main) {
        grid-area: contents;
    }
</style>
