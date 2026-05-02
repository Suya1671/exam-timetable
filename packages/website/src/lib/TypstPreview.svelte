<script lang='ts'>
    import setTypst from './setInitOptions'
    import '$lib/assets/typst.css'

    const typst = await setTypst()

    interface Props {
        vectorData: Uint8Array
    }

    const {
        vectorData,
    }: Props = $props()

    let previewCanvasContainer: HTMLDivElement | null = $state(null)

    $effect(async () => {
        if (!previewCanvasContainer)
            return

        await typst.canvas(previewCanvasContainer, { vectorData })
    })
</script>

<div bind:this={previewCanvasContainer}></div>
