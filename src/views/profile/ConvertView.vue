<script setup>
    import ProfilePage from '../../components/ProfilePage.vue';
    import ConvertPanel from '../../components/ConvertPanel.vue';
    import { ref, nextTick } from 'vue';

    // KiCad -> Altium. Two drop-in pairs (symbols, footprints); the converter's messages of
    // both go to the one log below. The library keeps the name of the input file / folder.
    const log = ref('')
    const logBox = ref(null)

    const append = async (text) =>
    {
        const time = new Date().toLocaleTimeString()
        log.value += `[${time}] ${text}\n`
        await nextTick()
        if (logBox.value)
        {
            logBox.value.scrollTop = logBox.value.scrollHeight
        }
    }
</script>

<template>
    <ProfilePage title="Convert">
        <onyks-text size="s">
            Converts KiCad libraries to Altium. Drop the files on the left box and the converted library
            appears on the right, ready to save. The library gets the name of the input file (or of the
            .pretty folder).
        </onyks-text>

        <ConvertPanel kind="symbols" title="Symbols (.kicad_sym → .SchLib)" extension=".kicad_sym"
            hint="One or more .kicad_sym files - each becomes its own .SchLib" @log="append"></ConvertPanel>

        <ConvertPanel kind="footprints" title="Footprints (.kicad_mod / .pretty → .PcbLib)" extension=".kicad_mod" with-folder
            hint="A .pretty folder, or .kicad_mod files - together they become one .PcbLib" @log="append"></ConvertPanel>

        <div class="logHeader">
            <span>Log</span>
            <onyks-button background="gray" @click="log = ''">Clear</onyks-button>
        </div>
        <textarea ref="logBox" class="log" readonly placeholder="Converter output" :value="log"></textarea>
    </ProfilePage>
</template>

<style scoped>
    .logHeader
    {
        width: 100%;
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-weight: 600;
    }

    .log
    {
        display: block;
        align-self: stretch;
        width: 100%;
        min-height: 360px;
        box-sizing: border-box;
        resize: vertical;
        padding: 12px;
        font-family: ui-monospace, Consolas, monospace;
        font-size: 13px;
        line-height: 1.45;
        color: inherit;
        background: transparent;
        border: 1px solid currentColor;
        border-radius: 8px;
        opacity: 0.85;
    }
</style>
