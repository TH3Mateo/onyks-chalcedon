import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useUserStore } from '../stores/user.js'
import { busy } from './useRepositoryActions.js'

// Keeps the local SVN working copy in sync automatically, two independent ways:
// - `autoupdate` + `autoupdateInterval`: periodic Pull (svn update), same pattern as
//   the DbLib sync in useDbLibSync.js.
// - `autoupdatePush`: alternative trigger - a native recursive folder watch (Rust
//   `start_watch`/`stop_watch`, see lib.rs) fires a `repository-changed` event the
//   moment any file under `repository.path` changes, which triggers a Push (commit).
// Both share the `busy` flag from useRepositoryActions.js so they never run at the
// same time as each other or as a manual Push/Pull from the UI.

const MIN_INTERVAL_SECONDS = 5

const status = ref({
    lastPullCheck: null,
    lastPush: null,
    error: '',
})

async function pullOnce()
{
    const userStore = useUserStore()
    if (busy.value)
    {
        return false
    }

    busy.value = true
    try
    {
        const exists = await invoke('is_svn_repository', { svnFolderPath: userStore.repository.path })
        if (exists)
        {
            await invoke('svn_update',
            {
                svnFolderPath: userStore.repository.path,
                login: userStore.login,
                password: userStore.password,
            })
        }
        else
        {
            await invoke('svn_checkout',
            {
                svnFolderPath: userStore.repository.path,
                login: userStore.login,
                password: userStore.password,
                url: userStore.repository.address,
            })
        }
        status.value.lastPullCheck = new Date()
        status.value.error = ''
        return true
    }
    catch (e)
    {
        status.value.error = String(e?.message ?? e)
        return false
    }
    finally
    {
        busy.value = false
    }
}

// Stops the watcher before pushing and restarts it after, so the commit's own file
// churn (updated working-copy timestamps, .svn metadata) never retriggers itself.
async function pushOnce()
{
    const userStore = useUserStore()
    if (busy.value)
    {
        return false
    }

    busy.value = true
    await invoke('stop_watch').catch(() => {})
    try
    {
        await invoke('svn_delete', { svnFolderPath: userStore.repository.path })
        await invoke('svn_add_all', { svnFolderPath: userStore.repository.path })
        await invoke('svn_commit',
        {
            svnFolderPath: userStore.repository.path,
            login: userStore.login,
            password: userStore.password,
            commitName: `Autopush from ${userStore.login} (ONYKS Chalcedon)`,
        })
        status.value.lastPush = new Date()
        status.value.error = ''
        return true
    }
    catch (e)
    {
        status.value.error = String(e?.message ?? e)
        return false
    }
    finally
    {
        busy.value = false
        if (userStore.repository.autoupdatePush && userStore.repository.path)
        {
            await invoke('start_watch', { path: userStore.repository.path }).catch(() => {})
        }
    }
}

let pullTimer = null
let started = false

// Called once from App.vue. Reschedules the Pull timer and restarts the folder watch
// whenever the related settings change.
export function startRepositorySync()
{
    if (started)
    {
        return
    }
    started = true

    const userStore = useUserStore()

    listen('repository-changed', () =>
    {
        if (userStore.repository.autoupdatePush)
        {
            pushOnce()
        }
    })

    watch(() => [
        userStore.isReady,
        userStore.repository.autoupdate,
        userStore.repository.autoupdateInterval,
        userStore.repository.path,
    ], () =>
    {
        clearInterval(pullTimer)
        pullTimer = null

        if (!userStore.isReady || !userStore.repository.autoupdate || !userStore.repository.path)
        {
            return
        }

        const seconds = Math.max(MIN_INTERVAL_SECONDS, Number(userStore.repository.autoupdateInterval) || MIN_INTERVAL_SECONDS)
        pullOnce()
        pullTimer = setInterval(pullOnce, seconds * 1000)
    }, { immediate: true })

    watch(() => [
        userStore.isReady,
        userStore.repository.autoupdatePush,
        userStore.repository.path,
    ], async () =>
    {
        await invoke('stop_watch').catch(() => {})

        if (userStore.isReady && userStore.repository.autoupdatePush && userStore.repository.path)
        {
            await invoke('start_watch', { path: userStore.repository.path }).catch((e) =>
            {
                status.value.error = String(e?.message ?? e)
            })
        }
    }, { immediate: true })
}

export function useRepositorySync()
{
    return { status, pullOnce, pushOnce }
}
