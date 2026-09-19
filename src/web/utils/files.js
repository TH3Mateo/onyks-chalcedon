import { fetch } from '@tauri-apps/plugin-http'
import { save } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { serverUrl } from './http.js'

// The webview cannot download files the way a browser does, so every "download" in the
// ported views asks for a target path and writes the file through the Rust side.

export async function saveBinaryFile(bytes, defaultName, filterName, extensions)
{
    const path = await save({ defaultPath: defaultName, filters: [{ name: filterName, extensions }] })
    if (!path)
    {
        return null
    }
    await invoke('write_binary_file', { path, data: Array.from(bytes) })
    return path
}

export async function saveRemoteFile(apiPath, defaultName, filterName, extensions)
{
    const response = await fetch(serverUrl(apiPath), { method: 'GET' })
    if (!response.ok)
    {
        throw new Error(`Request failed with status code ${response.status}`)
    }
    return saveBinaryFile(new Uint8Array(await response.arrayBuffer()), defaultName, filterName, extensions)
}
