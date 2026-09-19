<script setup lang="js">
    import WarningAlert from '@web/components/WarningAlert.vue';
    import { useRouter } from 'vue-router';
    const router = useRouter()
</script>

<template>
    <onyks-container type="stack" padding="l" gap="l"  justify="center">
        <warning-alert></warning-alert>
        <onyks-header>ERROR! PAGE NOT FOUND!</onyks-header>
        <!-- <img src="../../node_modules/onyks-web-ui-system/dist/logo.png" class="logo"> -->
        <onyks-text size="xl" class="message">Unfortunately! You get lost... Errors sometimes happen, but there is hope for better tomorrow. Only what you need to click the button below.</onyks-text>
        <onyks-button @click="router.push('/profile/web/dashboard')">Dashboard!</onyks-button>
    </onyks-container>
</template>

<style lang="css" scoped>
    onyks-header, onyks-alert, h1
    {
        width: 100%;
    }

    h1
    {
        text-align: center;
        flex: 1;
        font-size: 5rem;
    }

    .logo
    {
        width: 100%;
        height: auto;
        display: block;
        flex-shrink: 0;
    }

    .message
    {
        text-align: justify;
    }
</style>