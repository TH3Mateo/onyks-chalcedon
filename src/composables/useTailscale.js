import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUserStore } from '../stores/user.js'

// Connects this computer to the Tailscale network: starts the Tailscale client and
// runs `tailscale login --auth-key=<key>` with the key read from the text file chosen
// in Settings (Rust command `tailscale_connect`, see lib.rs).

const status = ref({
    running: false,
    lastConnect: null,
    message: '',
    error: '',
})

async function connectTailscale()
{
    const userStore = useUserStore()
    if (status.value.running)
    {
        return false
    }

    status.value.running = true
    try
    {
        status.value.message = await invoke('tailscale_connect',
        {
            authKeyPath: userStore.tailscale.authKeyPath,
        })
        status.value.lastConnect = new Date()
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
        status.value.running = false
    }
}

let started = false

// Called once from App.vue: connects once per program start, as soon as the settings
// are loaded and the option is on. Later changes in Settings do not reconnect by
// themselves - the "Connect now" button does that.
export function startTailscaleAutostart()
{
    if (started)
    {
        return
    }
    started = true

    const userStore = useUserStore()

    const stop = watch(() => userStore.isReady, (ready) =>
    {
        if (!ready)
        {
            return
        }
        stop?.()
        if (userStore.tailscale.autostart && userStore.tailscale.authKeyPath)
        {
            connectTailscale()
        }
    }, { immediate: true })
}

export function useTailscale()
{
    return { status, connectTailscale }
}
