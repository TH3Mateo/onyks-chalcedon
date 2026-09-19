<script setup>
    import { useRoute } from 'vue-router'
    import { ref, provide } from 'vue'
    import { readStorage, writeStorage } from '@web/utils/storage.js'
    import { useUserStore } from '../stores/user.js'
    const route = useRoute()
    const userStore = useUserStore()

    const fullWidth = ref(readStorage('onyks-full-width', false))
    const toggleFullWidth = () =>
    {
        fullWidth.value = !fullWidth.value
        writeStorage('onyks-full-width', fullWidth.value)
    }
    provide('fullWidth', fullWidth)
    provide('toggleFullWidth', toggleFullWidth)
</script>

<template>
    <div class="web-manager">
        <onyks-nav size="m" mobile-breakpoint="900" max-view-items="5">
            <img src="../assets/logo.png" class="logo">
            <router-link slot="nav" to="/profile/web/dashboard" .selected="route.path.endsWith('/dashboard')">Dashboard</router-link>
            <router-link slot="nav" to="/profile/web/management" .selected="route.path.endsWith('/management')">Management</router-link>
            <router-link slot="nav" to="/profile/web/repository" .selected="route.path.endsWith('/repository')">Repository</router-link>
            <router-link slot="nav" to="/profile/web/settings" .selected="route.path.endsWith('/settings')">Settings</router-link>
        </onyks-nav>
        <main :class="{ 'full-width': fullWidth }">
            <onyks-alert v-if="userStore.isReady && userStore.webManagerAddress.trim() == ''" type="error">
                The address of the web manager is unset. Fill it in the settings section.
            </onyks-alert>
            <router-view v-slot="{ Component, route }">
                <transition name="fade" mode="out-in">
                    <div :key="route.path">
                        <component :is="Component" />
                    </div>
                </transition>
            </router-view>
        </main>
    </div>
</template>

<style lang="css" scoped>
    .logo
    {
        height: 100%;
        padding: var(--onyks-spacing-md);
        box-sizing: border-box;
        position: absolute;
    }

    .web-manager
    {
        width: 100%;
    }

    onyks-nav
    {
        position: sticky;
        top: 0;
        margin: 0;
        width: auto;
        z-index: 12;
    }

    main
    {
        max-width: 1024px;
        margin-left: auto;
        margin-right: auto;
        box-sizing: border-box;
        transition: max-width 0.2s ease;
    }

    main.full-width
    {
        max-width: none;
        padding-left: var(--onyks-spacing-lg);
        padding-right: var(--onyks-spacing-lg);
    }

    .fade-enter-active, .fade-leave-active
    {
        transition: opacity 0.2s ease;
    }

    .fade-enter-from, .fade-leave-to
    {
        opacity: 0;
    }
</style>

<style lang="css">
    /* Global in Bloodstone; :where() keeps the specificity as low as the original element
       selector, so the per-view scoped overrides still win. */
    :where(.web-manager) onyks-button
    {
        width: 140px;
    }

    :where(.web-manager) onyks-dialog
    {
        z-index: 20;
        position: relative;
    }
</style>
