import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUserStore } from '../stores/user.js'
import { serverAddress, createClient } from '../web/utils/http.js'
import { detectTool } from './useCadTool.js'

// Creates the ODBC data source ("bloodstone") that the .DbLib / .kicad_dbl refer to, from the
// login and password of Settings and the database the server reports (/server/identity):
// the Altium account gets altium_lib, the KiCad one kicad_lib. The data source belongs to
// the current user (registry on Windows, ~/.odbc.ini on Linux) - that is where the password
// is kept, never in the library file.

export const ODBC_SOURCE_NAME = 'bloodstone'

const api = createClient('/api', { timeout: 5000 })

export const odbcStatus = ref({ message: '', error: '' })

let appliedKey = ''

// `force`: write even if the same values were already applied in this run.
export async function setupOdbcSource(force = false)
{
    const userStore = useUserStore()
    try
    {
        if (serverAddress() === '')
        {
            throw new Error('The address of the web manager is unset.')
        }
        const tool = await detectTool()
        const identity = (await api.get('/server/identity')).data
        const database = tool === 'kicad' ? identity.kicadDatabase : identity.altiumDatabase
        if (!identity.databaseHost || !database)
        {
            throw new Error('The server did not report its database (an older version of Bloodstone?).')
        }

        const key = [identity.databaseHost, identity.databasePort, database, userStore.login, userStore.password].join('|')
        if (!force && key === appliedKey)
        {
            return false
        }

        odbcStatus.value.message = await invoke('setup_odbc_source', {
            name: ODBC_SOURCE_NAME,
            host: identity.databaseHost,
            port: String(identity.databasePort || '5432'),
            database,
            username: userStore.login,
            password: userStore.password,
        })
        odbcStatus.value.error = ''
        appliedKey = key
        return true
    }
    catch (e)
    {
        odbcStatus.value.error = String(e?.response?.data?.detail ?? e?.message ?? e)
        return false
    }
}

export function useOdbcSource()
{
    return { status: odbcStatus, setupOdbcSource }
}
