<script setup>
    import ProfilePage from '../../components/ProfilePage.vue';
    import { onMounted, ref, inject, watch, nextTick } from 'vue';
    import { useUserStore } from '../../stores/user.js';
    import { useRepositoryActions } from '../../composables/useRepositoryActions.js';

    const userStore = useUserStore()
    const currentPath = ref(['\\'])
    const path = ref(null)
    const { run } = useRepositoryActions(inject('dialogs'))

    const handleClickBtn = async (source) =>
    {
        await run(source)
    }

    onMounted(() => 
    {
        watch(() => userStore.isReady, async (ready) => 
        {
            if (ready) 
            {
                await nextTick()

                if(userStore.repository.path != '')
                {
                    currentPath.value = [...userStore.repository.path.split('\\')]
                }

                path.value.disabled = true
            }
        }, { immediate: true })
    })
</script>

<template>
    <ProfilePage title="Repository">
        <onyks-path :content="currentPath" ref="path"></onyks-path>
        <onyks-container type="group" gap="m" padding="">
            <onyks-container type="stack" gap="m" class="files" padding="">
                <onyks-file-explorer ref="explorer"></onyks-file-explorer>
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