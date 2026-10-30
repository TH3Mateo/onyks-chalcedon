<script setup>
    import ProfilePage from '../../components/ProfilePage.vue';
    import { ref, computed } from 'vue';
    import { invoke } from '@tauri-apps/api/core';
    import { open } from '@tauri-apps/plugin-dialog';

    // KiCad -> Altium: .kicad_sym / .kicad_mod files or whole .pretty folders go in,
    // .SchLib / .PcbLib land in the output folder (converter bundled with the program).
    const symbols = ref([])
    const footprints = ref([])
    const outputDir = ref('')
    const name = ref('')
    const busy = ref(false)
    const log = ref('')
    const failed = ref(false)

    const asList = (value) => value === null ? [] : (Array.isArray(value) ? value : [value])

    const chooseSymbols = async () =>
    {
        const files = asList(await open({
            multiple: true,
            filters: [{ name: 'KiCad Symbol Library', extensions: ['kicad_sym'] }]
        }))
        if(files.length) symbols.value = files
    }

    const chooseFootprintFiles = async () =>
    {
        const files = asList(await open({
            multiple: true,
            filters: [{ name: 'KiCad Footprint', extensions: ['kicad_mod'] }]
        }))
        if(files.length) footprints.value = files
    }

    const chooseFootprintFolder = async () =>
    {
        const folder = await open({ directory: true, title: 'Select a .pretty folder' })
        if(folder) footprints.value = [folder]
    }

    const chooseOutput = async () =>
    {
        const folder = await open({ directory: true, title: 'Select the output folder' })
        if(folder) outputDir.value = folder
    }

    const canConvert = computed(() => !busy.value && outputDir.value !== '' && (symbols.value.length > 0 || footprints.value.length > 0))

    const convert = async () =>
    {
        busy.value = true
        failed.value = false
        log.value = 'Converting...'
        try
        {
            log.value = await invoke('convert_kicad_to_altium', {
                sym: symbols.value,
                fp: footprints.value,
                out: outputDir.value,
                name: name.value || null
            })
        }
        catch (e)
        {
            failed.value = true
            log.value = String(e)
        }
        finally
        {
            busy.value = false
        }
    }

    const clear = () =>
    {
        symbols.value = []
        footprints.value = []
        log.value = ''
        failed.value = false
    }

    const describe = (list) => list.join('\n')
</script>

<template>
    <ProfilePage title="Convert">
        <onyks-text size="s">
            Converts KiCad libraries to Altium: symbols (.kicad_sym) to a .SchLib and footprints
            (.kicad_mod files or a .pretty folder) to a .PcbLib. Python 3 has to be installed.
        </onyks-text>

        <onyks-header level="4">Input</onyks-header>

        <textarea class="box" readonly rows="3" placeholder="Symbol libraries (.kicad_sym)" :value="describe(symbols)"></textarea>
        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="yellow" @click="chooseSymbols">Select symbols</onyks-button>
        </onyks-container>

        <textarea class="box" readonly rows="3" placeholder="Footprints (.kicad_mod files or a .pretty folder)" :value="describe(footprints)"></textarea>
        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="yellow" @click="chooseFootprintFiles">Select footprint files</onyks-button>
            <onyks-button background="yellow" @click="chooseFootprintFolder">Select .pretty folder</onyks-button>
        </onyks-container>

        <onyks-header level="4">Output</onyks-header>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Output folder</onyks-text>
            <onyks-textfield size="m" v-model="outputDir" placeholder="e.g C:/Altium/libraries"></onyks-textfield>
            <onyks-text size="m">Library name (optional)</onyks-text>
            <onyks-textfield size="m" v-model="name" placeholder="defaults to the first input file name"></onyks-textfield>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="gray" @click="chooseOutput">Select folder</onyks-button>
            <onyks-button background="gray" @click="clear" :disabled="busy">Clear</onyks-button>
            <onyks-button background="green" @click="convert" :disabled="!canConvert">{{ busy ? 'Converting...' : 'Convert' }}</onyks-button>
        </onyks-container>

        <textarea class="box log" :class="{error: failed}" readonly rows="10" placeholder="Converter output" :value="log"></textarea>
    </ProfilePage>
</template>

<style scoped>
    .box
    {
        width: 100%;
        box-sizing: border-box;
        resize: vertical;
        padding: var(--onyks-spacing-md, 8px);
        font-family: monospace;
        font-size: 13px;
        color: inherit;
        background: transparent;
        border: 1px solid currentColor;
        border-radius: 6px;
        opacity: 0.85;
    }

    .box.error
    {
        color: #e5484d;
    }
</style>
