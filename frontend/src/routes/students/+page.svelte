<script lang="ts">
    import { db } from "$lib/db";
    import { Grid, Willow, type IColumnConfig } from "@svar-ui/svelte-grid";
    import { Button } from "m3-svelte";

    let students = await db.query.student.findMany({
        with: {
            subjects: true,
        },
    });

    const columns: (IColumnConfig & { id: keyof (typeof students)[number] })[] =
        [
            {
                id: "name",
                header: "Name",
            },
            {
                id: "grade",
                header: "Grade",
            },
        ];
</script>

<header>
    <h1>Students</h1>

    <Button onclick={() => alert("Not implemented yet")}>Import from CSV</Button
    >
</header>

<main>
    <Willow>
        <Grid data={students} {columns} filterValues={[]}></Grid>
    </Willow>
</main>
