import { fetch } from '@tauri-apps/plugin-http'
import { useUserStore } from '../../stores/user.js'

// The ported views were written against axios, so this keeps the part of its contract
// they rely on: a response is {status, data}, and a failed request rejects with an
// error carrying `response.data.detail` from FastAPI. Requests go through the Tauri
// HTTP plugin instead of the webview's fetch, which is not subject to CORS.

export function serverAddress()
{
    return useUserStore().webManagerAddress.trim().replace(/\/+$/, '')
}

export function serverUrl(path)
{
    return serverAddress() + path
}

class HttpError extends Error
{
    constructor(message, response = null)
    {
        super(message)
        this.response = response
        this.status = response?.status
    }
}

async function parseBody(response)
{
    const text = await response.text()
    try
    {
        return text === '' ? null : JSON.parse(text)
    }
    catch (e)
    {
        return text
    }
}

export function createClient(basePath, { timeout = 5000 } = {})
{
    const request = async (method, url, { params, data, headers } = {}) =>
    {
        if (serverAddress() === '')
        {
            throw new HttpError('The address of the web manager is unset. Fill it in the settings section.',
                { status: 0, data: { detail: 'The address of the web manager is unset. Fill it in the settings section.' } })
        }

        const query = new URLSearchParams()
        for (const [key, value] of Object.entries(params ?? {}))
        {
            if (value !== undefined && value !== null)
            {
                query.append(key, value)
            }
        }
        const queryString = query.toString()
        const fullUrl = serverUrl(basePath + url) + (queryString ? `?${queryString}` : '')

        const finalHeaders = { ...(headers ?? {}) }
        let body = undefined
        if (data instanceof FormData)
        {
            // The multipart boundary has to come from the body itself.
            delete finalHeaders['Content-Type']
            body = data
        }
        else if (data !== undefined)
        {
            finalHeaders['Content-Type'] = 'application/json'
            body = JSON.stringify(data)
        }

        let response = null
        try
        {
            response = await fetch(fullUrl, { method, headers: finalHeaders, body, connectTimeout: timeout })
        }
        catch (e)
        {
            throw new HttpError(String(e), { status: 0, data: { detail: `Cannot connect to the server: ${e}` } })
        }

        const result = { status: response.status, data: await parseBody(response) }
        if (!response.ok)
        {
            throw new HttpError(`Request failed with status code ${response.status}`, result)
        }
        return result
    }

    return {
        get: (url, config = {}) => request('GET', url, config),
        delete: (url, config = {}) => request('DELETE', url, config),
        post: (url, data, config = {}) => request('POST', url, { ...config, data }),
        put: (url, data, config = {}) => request('PUT', url, { ...config, data }),
    }
}
