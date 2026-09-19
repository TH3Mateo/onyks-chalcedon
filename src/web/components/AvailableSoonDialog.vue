<script setup>
    import { ref } from 'vue';

    const dialog = ref(null)

    const open = (parametr) => 
    {
        dialog.value.open = true
    };

    defineExpose({
        open
    });
</script>

<template>
    <onyks-dialog title="Information" scroll-target="body" modal corner-close ref="dialog" bottom-buttons>
        <onyks-text size="m">This feature will be available soon.</onyks-text>
        <onyks-button slot="footer" @click="dialog.open = false">Close</onyks-button>
    </onyks-dialog>             
</template>

<style>
    onyks-dialog
    {
        position: relative;
        z-index: 15;
    }
</style>