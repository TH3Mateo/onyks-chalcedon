import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUserStore } from '../stores/user.js'
import { serverAddress, createClient } from '../web/utils/http.js'
import { buildDbLib, normalize } from '../utils/dblib.js'

// Keeps the local Altium .DbLib file in step with the database: a new category table
// (e.g. batteries) gets its [TableN] section and field maps, a new supplier gets its
// code column mapped in every table. What the user configured in Altium stays - see
// utils/dblib.js for what is kept and what is generated.

const MIN_INTERVAL_SECONDS = 5
const PAGE_SIZE = 100

const api = createClient('/api', { timeout: 5000 })

const status = ref({
    running: false,
    lastCheck: null,
    lastChange: null,
    error: '',
})

// The list endpoints are paginated (at most 100 items per request).
async function fetchAll(url)
{
    const items = []
    for (let skip = 0; ; skip += PAGE_SIZE)
    {
        const response = await api.get(url, { params: { limit: PAGE_SIZE, skip } })
        items.push(...response.data.items)
        if (response.data.items.length < PAGE_SIZE || items.length >= response.data.total)
        {
            return items
        }
    }
}

export async function syncDbLib()
{
    const userStore = useUserStore()
    const path = userStore.database.dblibPath

    if (status.value.running)
    {
        return false
    }
    if (!path)
    {
        status.value.error = 'The path of the DbLib file is unset.'
        return false
    }
    if (serverAddress() === '')
    {
        status.value.error = 'The address of the web manager is unset.'
        return false
    }

    status.value.running = true
    try
    {
        // Ordered by id, so new categories are appended after the existing ones.
        const tables = (await fetchAll('/table/list')).map((table) => table.name)
        // Not ordered by the server; sorted so the file does not change just because
        // the database returned the rows in a different order.
        const suppliers = (await fetchAll('/supplier/list')).sort((a, b) => a.id - b.id)

        const existing = await invoke('read_text_file', { path })
        const content = buildDbLib(existing, tables, suppliers)
        const changed = existing === null || normalize(existing) !== normalize(content)

        if (changed)
        {
            await invoke('write_text_file', { path, content })
            status.value.lastChange = new Date()
        }

        status.value.lastCheck = new Date()
        status.value.error = ''
        return changed
    }
    catch (e)
    {
        status.value.error = String(e?.response?.data?.detail ?? e?.message ?? e)
        return false
    }
    finally
    {
        status.value.running = false
    }
}

let timer = null
let started = false

// Called once from App.vue. Reschedules itself whenever the related settings change.
export function startDbLibAutoupdate()
{
    if (started)
    {
        return
    }
    started = true

    const userStore = useUserStore()

    watch(() => [
        userStore.isReady,
        userStore.database.autoupdate,
        userStore.database.autoupdateInterval,
        userStore.database.dblibPath,
        userStore.webManagerAddress,
    ], () =>
    {
        clearInterval(timer)
        timer = null

        if (!userStore.isReady || !userStore.database.autoupdate || !userStore.database.dblibPath)
        {
            return
        }

        const seconds = Math.max(MIN_INTERVAL_SECONDS, Number(userStore.database.autoupdateInterval) || MIN_INTERVAL_SECONDS)
        syncDbLib()
        timer = setInterval(syncDbLib, seconds * 1000)
    }, { immediate: true })
}

export function useDbLibSync()
{
    return { status, syncDbLib }
}
