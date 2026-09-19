<script setup>
    import { useWindowSize } from '@vueuse/core';
    const { width} = useWindowSize()
</script>

<template>
    <onyks-container type="grid" padding="" gap="l" :cols="width > 1000? 6:(width > 500? 3:2)" class="btns">
        <slot></slot>
    </onyks-container>
</template>

<style>
    .btns > onyks-button
    {
        width: 100%;
    }
</style>