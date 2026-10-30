import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { fetch } from '@tauri-apps/plugin-http'

// Finds the ONYKS server without typing its IP address: the Rust command
// `discover_servers` (lib.rs) probes the web manager's port in the local networks of
// this computer (LAN, Wi-Fi, VPN adapters) and among the Tailscale peers, then each
// computer found is asked for /api/server/identity to tell the real server from
// any other service listening on that port.

const DEFAULT_PORT = 8113
const VERIFY_TIMEOUT_MS = 3000

const status = ref({
    running: false,
    searched: false,
    servers: [],   // [{ ip, address, verified }]
    error: '',
})

export function portOf(address)
{
    try
    {
        return Number(new URL(address).port) || DEFAULT_PORT
    }
    catch (e)
    {
        return DEFAULT_PORT
    }
}

async function isOnyksServer(address)
{
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), VERIFY_TIMEOUT_MS)
    try
    {
        const response = await fetch(`${address}/api/server/identity`, { signal: controller.signal })
        if (!response.ok)
        {
            return false
        }
        const body = await response.json()
        return body !== null && typeof body === 'object'
    }
    catch (e)
    {
        return false
    }
    finally
    {
        clearTimeout(timer)
    }
}

async function discoverServers(port = DEFAULT_PORT)
{
    if (status.value.running)
    {
        return
    }

    status.value.running = true
    status.value.error = ''
    try
    {
        const ips = await invoke('discover_servers', { port })
        const servers = await Promise.all(ips.map(async (ip) =>
        {
            const address = `http://${ip}:${port}`
            return { ip, address, verified: await isOnyksServer(address) }
        }))
        // The real servers first.
        status.value.servers = servers.sort((a, b) => Number(b.verified) - Number(a.verified))
        status.value.searched = true
    }
    catch (e)
    {
        status.value.error = String(e?.message ?? e)
    }
    finally
    {
        status.value.running = false
    }
}

// SVN repository of the ONYKS server: same machine as the web manager, fixed port and name.
export const SVN_PORT = 8111
export const SVN_REPOSITORY = 'part_library'

export function defaultRepositoryAddress(ip)
{
    return `http://${ip}:${SVN_PORT}/svn/${SVN_REPOSITORY}`
}

// Same address with the host replaced (the port, path and scheme stay), e.g. an SVN
// repository address after the server got a new IP. Returns the input when it is not
// a valid URL.
export function replaceHost(address, ip)
{
    try
    {
        const url = new URL(address)
        url.hostname = ip
        return url.toString()
    }
    catch (e)
    {
        return address
    }
}

export function useServerDiscovery()
{
    return { status, discoverServers }
}
