<script setup>
    import { ref, computed } from 'vue';
    import { invoke } from '@tauri-apps/api/core';
    import { open, save } from '@tauri-apps/plugin-dialog';

    // One "input -> output" pair of the Convert page. Files are dropped on the input box (or
    // picked with the box / the folder button) - no paths are typed. They are read in the
    // window and converted at once; the results appear in the output box and are saved
    // from there. kind: 'symbols' (.kicad_sym -> .SchLib) or 'footprints' (.kicad_mod / a
    // .pretty folder -> .PcbLib).
    const props = defineProps({
        kind: { type: String, required: true },
        title: { type: String, required: true },
        extension: { type: String, required: true },   // '.kicad_sym' | '.kicad_mod'
        hint: { type: String, required: true },
        withFolder: { type: Boolean, default: false },
    })
    const emit = defineEmits(['log'])

    const inputs = ref([])      // names shown in the input box
    const outputs = ref([])     // [{ name, data }] data = base64
    const busy = ref(false)
    const dragging = ref(false)
    const picker = ref(null)
    const folderPicker = ref(null)

    const stem = (name) => name.replace(/\.[^.]*$/, '')
    const hasExtension = (name) => name.toLowerCase().endsWith(props.extension)

    // Files of a dropped folder, found recursively. Must be started inside the drop event.
    const readEntry = (entry) => new Promise((resolve) =>
    {
        if (entry.isFile)
        {
            entry.file((file) => resolve([file]), () => resolve([]))
        }
        else if (entry.isDirectory)
        {
            const reader = entry.createReader()
            const all = []
            const next = () => reader.readEntries(async (batch) =>
            {
                if (batch.length === 0)
                {
                    const nested = await Promise.all(all.map(readEntry))
                    resolve(nested.flat())
                    return
                }
                all.push(...batch)
                next()
            }, () => resolve([]))
            next()
        }
        else
        {
            resolve([])
        }
    })

    const onDrop = async (event) =>
    {
        dragging.value = false
        const items = [...(event.dataTransfer?.items ?? [])]
        const entries = items.map((item) => item.webkitGetAsEntry?.()).filter(Boolean)
        if (entries.length === 0)
        {
            return
        }
        // [{ file, folder }] - `folder` is the name of the dropped folder the file came from
        const found = []
        for (const entry of entries)
        {
            const files = await readEntry(entry)
            files.forEach((file) => found.push({ file, folder: entry.isDirectory ? entry.name : null }))
        }
        await convert(found)
    }

    const onPick = async (event, relative) =>
    {
        const files = [...event.target.files]
        event.target.value = ''
        await convert(files.map((file) => ({
            file,
            folder: relative ? file.webkitRelativePath.split('/')[0] : null,
        })))
    }

    const convert = async (found) =>
    {
        const matching = found.filter(({ file }) => hasExtension(file.name))
        if (matching.length === 0)
        {
            emit('log', `${props.title}: no ${props.extension} files in what was dropped.`)
            return
        }

        busy.value = true
        outputs.value = []
        inputs.value = matching.map(({ file }) => file.name)
        try
        {
            const read = await Promise.all(matching.map(async ({ file, folder }) => ({ name: file.name, folder, content: await file.text() })))

            // Symbols: one library per file, named like the file. Footprints: one library
            // named like the dropped .pretty folder, or like the first file when loose.
            const jobs = props.kind === 'symbols'
                ? read.map((file) => ({ name: stem(file.name), files: [file] }))
                : [{ name: read[0].folder ? read[0].folder.replace(/\.pretty$/i, '') : stem(read[0].name), files: read }]

            for (const job of jobs)
            {
                emit('log', `> ${props.title}: ${job.name} (${job.files.length} file${job.files.length === 1 ? '' : 's'})`)
                try
                {
                    const result = await invoke('convert_kicad_content', {
                        kind: props.kind,
                        name: job.name,
                        files: job.files.map(({ name, content }) => ({ name, content })),
                    })
                    emit('log', result.log || (result.ok ? 'Done.' : 'The converter produced no file.'))
                    outputs.value.push(...result.outputs)
                }
                catch (e)
                {
                    emit('log', `ERROR: ${String(e?.message ?? e)}`)
                }
            }
        }
        finally
        {
            busy.value = false
        }
    }

    const bytesOf = (base64) => Uint8Array.from(atob(base64), (c) => c.charCodeAt(0))

    const saveOne = async (output) =>
    {
        const ext = output.name.split('.').pop()
        const path = await save({
            defaultPath: output.name,
            filters: [{ name: 'Altium library', extensions: [ext] }],
        })
        if (path)
        {
            await invoke('write_binary_file', { path, data: Array.from(bytesOf(output.data)) })
            emit('log', `Saved ${path}`)
        }
    }

    const saveAll = async () =>
    {
        const folder = await open({ directory: true, title: 'Select the folder for the converted libraries' })
        if (folder)
        {
            for (const output of outputs.value)
            {
                const path = `${String(folder).replace(/[\\/]+$/, '')}/${output.name}`
                await invoke('write_binary_file', { path, data: Array.from(bytesOf(output.data)) })
                emit('log', `Saved ${path}`)
            }
        }
    }

    const sizeOf = (output) => `${Math.max(1, Math.round(output.data.length * 0.75 / 1024))} KB`
    const inputText = computed(() => inputs.value.join('\n'))
</script>

<template>
    <div class="pair">
        <div class="title">{{ title }}</div>
        <div class="boxes">
            <div class="zone input" :class="{ dragging, busy }"
                @dragenter.prevent="dragging = true" @dragover.prevent="dragging = true"
                @dragleave.prevent="dragging = false" @drop.prevent="onDrop"
                @click="picker.click()">
                <template v-if="inputs.length === 0">
                    <div class="big">Drop here</div>
                    <div class="small">{{ hint }}</div>
                    <div class="small">or click to choose files</div>
                </template>
                <pre v-else class="list">{{ inputText }}</pre>
                <input ref="picker" type="file" multiple :accept="extension" hidden @click.stop @change="onPick($event, false)">
                <input v-if="withFolder" ref="folderPicker" type="file" webkitdirectory hidden @click.stop @change="onPick($event, true)">
            </div>

            <div class="zone output" :class="{ busy }">
                <div v-if="busy" class="big">Converting...</div>
                <div v-else-if="outputs.length === 0" class="small">The converted Altium libraries will appear here.</div>
                <div v-for="output in outputs" :key="output.name" class="file">
                    <span class="name">{{ output.name }}</span>
                    <span class="size">{{ sizeOf(output) }}</span>
                    <onyks-button background="green" @click="saveOne(output)">Save</onyks-button>
                </div>
                <onyks-button v-if="outputs.length > 1" background="blue" @click="saveAll">Save all to folder</onyks-button>
            </div>
        </div>
        <div v-if="withFolder" class="actions">
            <onyks-button background="yellow" @click="folderPicker.click()">Choose .pretty folder</onyks-button>
        </div>
    </div>
</template>

<style scoped>
    .pair
    {
        width: 100%;
    }

    .title
    {
        font-weight: 600;
        margin-bottom: 8px;
    }

    .boxes
    {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 16px;
    }

    .zone
    {
        box-sizing: border-box;
        min-height: 150px;
        padding: 12px;
        border: 2px dashed currentColor;
        border-radius: 10px;
        opacity: 0.8;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        gap: 8px;
        text-align: center;
        overflow: auto;
    }

    .input
    {
        cursor: pointer;
    }

    .input.dragging
    {
        opacity: 1;
        border-style: solid;
        background: rgba(128, 128, 128, 0.15);
    }

    .output
    {
        border-style: solid;
        justify-content: flex-start;
    }

    .busy
    {
        opacity: 0.5;
    }

    .big
    {
        font-size: 18px;
        font-weight: 600;
    }

    .small
    {
        font-size: 13px;
        opacity: 0.7;
    }

    .list
    {
        margin: 0;
        font-size: 13px;
        text-align: left;
        align-self: stretch;
    }

    .file
    {
        display: flex;
        align-items: center;
        gap: 12px;
        align-self: stretch;
    }

    .file .name
    {
        flex: 1;
        text-align: left;
        word-break: break-all;
    }

    .file .size
    {
        font-size: 12px;
        opacity: 0.7;
    }

    .actions
    {
        margin-top: 8px;
        display: flex;
        justify-content: flex-end;
    }
</style>
