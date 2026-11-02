import { ref } from 'vue'
import { useUserStore } from '../stores/user.js'
import { fetch } from '@tauri-apps/plugin-http'
import { serverAddress, createClient } from '../web/utils/http.js'

// Which CAD tool the account from Settings uses ('altium' or 'kicad'). Only the server
// knows it (private.users.tool), so it is asked with the same login and password that
// are used for SVN: POST /api/user/identify. The answer is cached until the login, the
// password or the server address change.

const api = createClient('/api', { timeout: 5000 })

export const tool = ref(null)

let cachedKey = ''

export async function detectTool()
{
    const userStore = useUserStore()
    const key = `${serverAddress()}|${userStore.login}|${userStore.password}`

    if (tool.value !== null && key === cachedKey)
    {
        return tool.value
    }
    if (!userStore.login || !userStore.password)
    {
        throw new Error('The login and password are unset (needed to find out which CAD tool the account uses).')
    }

    try
    {
        const response = await api.post('/user/identify', { login: userStore.login, password: userStore.password })
        tool.value = response.data.tool === 'kicad' ? 'kicad' : 'altium'
    }
    catch (e)
    {
        const status = e?.response?.status
        if (status === 404 || status === 405)
        {
            // A server from before the endpoint existed: Altium was the only tool then.
            tool.value = 'altium'
        }
        else
        {
            tool.value = null
            throw e
        }
    }

    cachedKey = key
    return tool.value
}

// Asks the server at `address` whether the login and password are right (the server finds
// the same answer for SVN and the database). 'ok' | 'invalid' | 'unknown' - unknown when the
// server cannot say (an older version without the endpoint, no connection).
export async function checkCredentials(address, login, password)
{
    if (!address || !login || !password)
    {
        return 'unknown'
    }
    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), 5000)
    try
    {
        const response = await fetch(`${address}/api/user/identify`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ login, password }),
            signal: controller.signal,
        })
        if (response.ok)
        {
            return 'ok'
        }
        return response.status === 401 ? 'invalid' : 'unknown'
    }
    catch (e)
    {
        return 'unknown'
    }
    finally
    {
        clearTimeout(timer)
    }
}
