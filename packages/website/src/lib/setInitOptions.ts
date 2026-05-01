import rendererWasmUrl from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url'
import compilerWasmUrl from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url'
// Prevents reinitialization of compiler and renderer options during HMR (Hot Module Replacement).
// Use prepareUseOnce flag ensures initialization occurs only once to avoid duplicate calls to setXXXInitOptions.
// From https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.svelte/src/lib/set-init-options-typst.ts
import { $typst, loadFonts } from '@myriaddreamin/typst.ts'

let inited = false

export default async () => {
    if (!inited) {
        const fontsImport = import.meta.glob<false, string, string>('$lib/assets/GoogleSansFlex/*.ttf', { query: '?url', import: 'default' })
        const fonts = await Promise.all(Object.values(fontsImport).map(async fn => fn()))

        $typst.setCompilerInitOptions({
            getModule: () =>
                compilerWasmUrl,
            beforeBuild: [
                loadFonts(fonts),
            ],
        })

        $typst.setRendererInitOptions({
            getModule: () => rendererWasmUrl,
            beforeBuild: [
                loadFonts(fonts),
            ],
        })

        inited = true
    }
    return $typst
}
