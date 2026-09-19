<script setup>
    import { defineModel } from 'vue'
    import { useWindowSize } from '@vueuse/core'

    const model = defineModel()
    const {width} = useWindowSize()
    const props = defineProps(['table'])
</script>

<template>
    <onyks-container type="grid" :cols="width > 1100 ? 4 : width > 700 ? 3 : 2" padding="" gap="s" >
        
        <onyks-container v-for="(column, index) in model" :key="index" padding="" gap="s" type="group" align="center">

            <onyks-checkbox
                :checked="!column.hidden"
                size="s"
                @change="(e) => {column.hidden = !e.target.checked; props.table.refresh()}">
            </onyks-checkbox>

            <onyks-text size="s">{{ column.label }}</onyks-text>
        </onyks-container>
    </onyks-container>
</template>

<style scoped>
</style>