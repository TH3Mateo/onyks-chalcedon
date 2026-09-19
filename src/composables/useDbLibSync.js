import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { fetch } from '@tauri-apps/plugin-http'
import { useUserStore } from '../stores/user.js'
import { serverAddress, serverUrl } from '../web/utils/http.js'

// Keeps the local Altium .DbLib file in step with the database. The server generates
// the file from the current state of the database (every category table becomes a
// [TableN] section with its field maps), so a newly added table - e.g. batteries -
// shows up here on the next check without anyone downloading the file by hand.

const MIN_INTERVAL_MINUTES = 1

const status = ref({
    running: false,
    lastCheck: null,
    lastChange: null,
    error: '',
})

const normalize = (text) => text.replace(/\r\n/g, '\n')

const connectionStringLine = (text) =>
    normalize(text).split('\n').find((line) => line.startsWith('ConnectionString=')) ?? null

// The generated file leaves User ID and Password blank. The login is filled in from the
// settings; the password is never written by the program. Once the file exists, its
// connection string is kept as it is, so a password the user typed into it (or any
// other change to the connection) survives every later update.
function personalize(generated, existing, login)
{
    const existingConnection = existing === null ? null : connectionStringLine(existing)
    return normalize(generated).split('\n').map((line) =>
    {
        if (!line.startsWith('ConnectionString='))
        {
            return line
        }
        if (existingConnection !== null)
        {
            return existingConnection
        }
        return line.replace('User ID=;', `User ID=${login};`)
    }).join('\n')
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
        const response = await fetch(serverUrl('/api/settings/dblib'), { method: 'GET', connectTimeout: 5000 })
        if (!response.ok)
        {
            throw new Error(`The server answered with status code ${response.status}.`)
        }

        const existing = await invoke('read_text_file', { path })
        const content = personalize(await response.text(), existing, userStore.login)
        const changed = existing === null || normalize(existing) !== content

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
        status.value.error = String(e?.message ?? e)
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

        const minutes = Math.max(MIN_INTERVAL_MINUTES, Number(userStore.database.autoupdateInterval) || MIN_INTERVAL_MINUTES)
        syncDbLib()
        timer = setInterval(syncDbLib, minutes * 60 * 1000)
    }, { immediate: true })
}

export function useDbLibSync()
{
    return { status, syncDbLib }
}
