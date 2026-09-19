<script setup>
    import { ref, provide } from 'vue';
    import { useRoute } from 'vue-router';
    import AboutProgramDialog from '../components/AboutProgramDialog.vue';
    import ErrorDialog from '../components/ErrorDialog.vue';
    import ProgressDialog from '../components/ProgressDialog.vue';
    import { useRepositoryActions } from '../composables/useRepositoryActions.js';

    const route = useRoute()
    const dialogs = ref({about: null, error: null, progress: null})
    provide('dialogs', dialogs)

    const { run: repositoryAction, busy: repositoryBusy } = useRepositoryActions(dialogs)

    // Every page of the web manager shares one WebManagerLayout instance; keying by the
    // full path would remount it (and its navigation bar) on every click inside it.
    const viewKey = (route) => route.matched[1]?.path ?? route.fullPath
</script>

<template>
    <onyks-container type="group" gap="" padding="" class="container">
        <onyks-container padding="l" style="padding-right: 0;">
            <onyks-strip-menu type="v">
                <RouterLink to="/profile/web" title="Web Manager"><onyks-strip-menu-option size="m" icon="F43C" :selected="route.path.startsWith('/profile/web')"></onyks-strip-menu-option></RouterLink>
                <RouterLink to="/profile/repository" title="Repository"><onyks-strip-menu-option size="m" icon="F10D" :selected="route.path == '/profile/repository'"></onyks-strip-menu-option></RouterLink>
                <RouterLink to="/profile/settings" title="Settings"><onyks-strip-menu-option size="m" icon="F3E3" :selected="route.path == '/profile/settings'"></onyks-strip-menu-option></RouterLink>
                <onyks-strip-menu-option size="m" icon="F297" title="Push the repository" :class="{busy: repositoryBusy}" @click="repositoryAction('push')"></onyks-strip-menu-option>
                <onyks-strip-menu-option size="m" icon="F295" title="Pull the repository" :class="{busy: repositoryBusy}" @click="repositoryAction('pull')"></onyks-strip-menu-option>
                <onyks-strip-menu-option size="m" icon="F431" title="About the program" @click="dialogs.about.open"></onyks-strip-menu-option>
            </onyks-strip-menu>
        </onyks-container>
        <onyks-container class="content" padding='' gap="m" style="overflow-y: auto;">
            <Transition name="fade" mode="out-in" appear>
                <router-view v-slot="{ Component, route }">
                    <component :is="Component" :key="viewKey(route)" />
                </router-view>
            </Transition>
        </onyks-container>
    </onyks-container>

    <AboutProgramDialog :ref="(el) => {if(dialogs && el) dialogs.about = el}"></AboutProgramDialog>
    <ErrorDialog :ref="(el) => {if(dialogs && el) dialogs.error = el}"></ErrorDialog>
    <ProgressDialog :ref="(el) => {if(dialogs && el) dialogs.progress = el}"></ProgressDialog>
</template>

<style lang="css" scoped>
    .container
    {
        height: 100vh;
        box-sizing: border-box;
    }

    onyks-strip-menu
    {
        height: 100%;
        box-sizing: border-box;
    }

    onyks-strip-menu-option.busy
    {
        opacity: 0.5;
        pointer-events: none;
    }

    .content
    {
        flex: 1;
        box-sizing: border-box;
        height: 100%;
    }

    .fade-enter-active, .fade-leave-active
    {
        transition: opacity 0.5s ease;
    }

    .fade-enter-from, .fade-leave-to
    {
        opacity: 0;
    }

    a
    {
        color: inherit;
        text-decoration: none;
    }
</style>
