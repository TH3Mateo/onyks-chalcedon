import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from '@tauri-apps/plugin-dialog'
import { useUserStore } from '../stores/user.js'
import { KICAD_DBL_FILE } from '../utils/libraryFiles.js'
import { detectTool } from './useCadTool.js'

// Sets KiCad up for an account that uses KiCad (Rust command `kicad_sync`, see kicad.rs):
// the ONYKS_LIBRARY path variable in kicad_common.json, and the ONYKS rows of the global
// symbol and footprint library tables, copied from kicad_tables/ of the SVN working copy
// (which the server generates) plus the database library. Runs at start, after every Pull and
// after the .kicad_dbl is written. Nothing is done for an Altium account.

// KiCad's configuration folder is named after its version; change it here for another one.
export const KICAD_VERSION = '9.0'

const status = ref({
    active: false,      // the account uses KiCad
    running: false,
    lastSync: null,
    configDir: '',
    library: '',
    symbols: 0,
    footprints: 0,
    database: false,
    notes: [],
    error: '',
})

export async function syncKicad()
{
    const userStore = useUserStore()
    if (status.value.running || !userStore.isReady || !userStore.repository.path)
    {
        return false
    }

    status.value.running = true
    try
    {
        const tool = await detectTool()
        status.value.active = tool === 'kicad'
        if (tool !== 'kicad')
        {
            return false
        }

        const result = await invoke('kicad_sync',
        {
            workingDir: userStore.repository.path,
            version: KICAD_VERSION,
            databaseFile: KICAD_DBL_FILE,
        })
        Object.assign(status.value, {
            configDir: result.configDir,
            library: result.library,
            symbols: result.symbols,
            footprints: result.footprints,
            database: result.database,
            notes: result.notes,
            lastSync: new Date(),
            error: '',
        })

        // KiCad reads the tables when an editor starts; a running one has to be reopened.
        if (result.changed && result.kicadRunning)
        {
            message('The ONYKS libraries in KiCad were updated. Close and reopen KiCad to see them.',
                { title: 'ONYKS Chalcedon', kind: 'info' }).catch(() => {})
        }
        return result.changed
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

let started = false

// Called once from App.vue: synchronizes as soon as the settings are loaded, and again when
// the library folder or the login changes.
export function startKicadSync()
{
    if (started)
    {
        return
    }
    started = true

    const userStore = useUserStore()
    watch(() => [userStore.isReady, userStore.repository.path, userStore.login, userStore.password, userStore.webManagerAddress],
        () => { syncKicad() }, { immediate: true })
}

export function useKicadSync()
{
    return { status, syncKicad }
}
