import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import { useUserStore } from '../stores/user.js'

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms))

// Shared between the Repository page, the Push/Pull buttons of the strip menu, and the
// background autoupdate sync (useRepositorySync.js), so only one SVN operation runs at
// a time no matter where it was started from.
export const busy = ref(false)

// `dialogs` is a ref of {error, progress} holding the ErrorDialog and ProgressDialog
// instances that report the progress of the operation.
export function useRepositoryActions(dialogs)
{
    const userStore = useUserStore()

    const progress = (message, state) =>
    {
        dialogs.value.progress.message = message
        dialogs.value.progress.state = state
    }

    const showError = (message) =>
    {
        dialogs.value.error.message = message
        dialogs.value.error.open()
    }

    const finish = async (message) =>
    {
        progress(message, 100)
        await sleep(1000)
        dialogs.value.progress.toggleOpen(false)
    }

    const fail = async (e) =>
    {
        console.log(e)
        await finish('Error...')
        showError(String(e))
        return false
    }

    const repositoryIsExist = async () =>
    {
        progress('Checking the state of the repository...', 25)
        dialogs.value.progress.toggleOpen(true)
        await sleep(1000)
        return await invoke('is_svn_repository', {svnFolderPath: userStore.repository.path})
    }

    const repositoryPull = async (exists) =>
    {
        progress('Downloading the content of the repository...', 50)
        await sleep(1000)
        try
        {
            if(exists)
            {
                await invoke('svn_update',
                {
                    svnFolderPath: userStore.repository.path,
                    login: userStore.login,
                    password: userStore.password
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
            await finish('Finishing...')
            return true
        }
        catch(e)
        {
            return await fail(e)
        }
    }

    const repositoryPush = async () =>
    {
        progress('Commiting the changes...', 50)
        await sleep(1000)
        try
        {
            await invoke('svn_delete', { svnFolderPath: userStore.repository.path })
            await invoke('svn_add_all', { svnFolderPath: userStore.repository.path })
            await invoke('svn_commit',
            {
                svnFolderPath: userStore.repository.path,
                login: userStore.login,
                password: userStore.password,
                commitName: `Changes from ${userStore.login} (ONYKS Chalcedon)`
            })
            await finish('Finishing...')
            return true
        }
        catch(e)
        {
            return await fail(e)
        }
    }

    const repositoryCleanup = async () =>
    {
        progress('Cleaning repository...', 50)
        await sleep(1000)
        try
        {
            await invoke('svn_cleanup', { svnFolderPath: userStore.repository.path })
            await finish('Finishing...')
            return true
        }
        catch(e)
        {
            return await fail(e)
        }
    }

    const repositoryRevert = async () =>
    {
        progress('Reverting changes...', 50)
        await sleep(1000)
        try
        {
            await invoke('svn_revert', { svnFolderPath: userStore.repository.path })
            await finish('Finishing...')
            return true
        }
        catch(e)
        {
            return await fail(e)
        }
    }

    const run = async (action) =>
    {
        if(userStore.repository.path == '')
        {
            showError('The repository path is empty. Fill it in the settings section.')
            return false
        }

        if(action == 'explorer')
        {
            await openPath(userStore.repository.path)
            return true
        }

        if(busy.value)
        {
            showError('Another repository operation is still running.')
            return false
        }

        busy.value = true
        try
        {
            switch(action)
            {
                case 'push':
                    await repositoryIsExist()
                    return await repositoryPush()
                case 'pull':
                    return await repositoryPull(await repositoryIsExist())
                case 'revert':
                    await repositoryIsExist()
                    return await repositoryRevert()
                case 'reset':
                    await repositoryIsExist()
                    return await repositoryCleanup()
            }
        }
        finally
        {
            busy.value = false
        }
    }

    return { run, busy }
}
