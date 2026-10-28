<script setup>
    import ProfilePage from '../../components/ProfilePage.vue';
    import { onMounted, onUnmounted, ref, inject, watch, nextTick } from 'vue';
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
    import { useUserStore } from '../../stores/user.js';
    import { useRepositoryActions } from '../../composables/useRepositoryActions.js';

    const userStore = useUserStore()
    const currentPath = ref([])
    const path = ref(null)
    const explorer = ref(null)
    const error = ref('')
    const { run } = useRepositoryActions(inject('dialogs'))

    // The path bar cannot leave the repository folder: rootLength segments are fixed.
    let rootLength = 0
    let unlisten = null

    const splitPath = (value) => value.split(/[\\/]/)

    const load = async () =>
    {
        error.value = ''
        try
        {
            // Joined with "/", which Windows accepts too; a leading "" keeps the "/" of Linux paths.
            explorer.value.content = await invoke('list_directory', { path: path.value.content.join('/') || '/' })
        }
        catch (e)
        {
            explorer.value.content = []
            error.value = String(e)
        }
    }

    const openRoot = async () =>
    {
        const root = userStore.repository.path
        if(!root)
        {
            currentPath.value = []
            explorer.value.content = []
            return
        }
        const segments = splitPath(root.replace(/[\\/]+$/, ''))
        rootLength = segments.length
        currentPath.value = segments
        await nextTick()
        await load()
    }

    const pathChange = async () =>
    {
        if (path.value.content.length < rootLength)
        {
            path.value.content = splitPath(userStore.repository.path.replace(/[\\/]+$/, ''))
        }
        await load()
    }

    const enterFolder = async (e) =>
    {
        path.value.content = [...path.value.content, e.detail.folder.name]
        await load()
    }

    const handleClickBtn = async (source) =>
    {
        await run(source)
        if (path.value) await load()
    }

    onMounted(async () =>
    {
        explorer.value.types = [{type: 'file', icon: 'F390', isLikeDir: false},
        {type: 'dir', icon: 'F3D8', isLikeDir: true}]
        explorer.value.emptyAlert = userStore.repository.path ? 'The folder is empty' : 'No repository folder is set - choose it in Settings.'

        watch(() => userStore.isReady, async (ready) =>
        {
            if (ready) await openRoot()
        }, { immediate: true })

        // Refresh when the folder watcher reports a change (pull, push, edits in other programs).
        unlisten = await listen('repository-changed', () => { if (path.value) load() })
    })

    onUnmounted(() => unlisten?.())
</script>

<template>
    <ProfilePage title="Repository">
        <onyks-path :content="currentPath" ref="path" @path-change="pathChange"></onyks-path>
        <onyks-alert v-if="error" type="error">{{ error }}</onyks-alert>
        <onyks-container type="group" gap="m" padding="">
            <onyks-container type="stack" gap="m" class="files" padding="">
                <onyks-file-explorer ref="explorer" @enter-folder="enterFolder"></onyks-file-explorer>
            </onyks-container>
            <onyks-container type="stack" class="btns" gap="m" padding="">
                <onyks-button background="green" @click="handleClickBtn('push')">Push</onyks-button>
                <onyks-button background="blue" @click="handleClickBtn('pull')">Pull</onyks-button>
                <onyks-button background="red" @click="handleClickBtn('reset')">Reset</onyks-button>
                <onyks-button background="yellow" @click="handleClickBtn('revert')">Revert</onyks-button>
                <onyks-button background="gray" @click="handleClickBtn('explorer')">Explorer</onyks-button>
            </onyks-container>
        </onyks-container>
    </ProfilePage>
</template>

<style scoped>
    onyks-file-explorer
    {
        width: 100%;
    }

    onyks-path
    {
        width: 100%;
    }

    .btns
    {
        width: 140px;
    }

    .btns > onyks-button
    {
        width: 100%;
    }

    .files
    {
        flex: 1;
    }
</style>