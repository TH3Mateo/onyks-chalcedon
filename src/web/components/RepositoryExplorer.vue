<script setup lang="js">
    import { onMounted } from 'vue';
    import { ref } from 'vue';
    import { repository } from '@web/utils/api';
    import { defineExpose } from 'vue';
    
    const path = ref(null)
    const explorer = ref(null)
    const props = defineProps(
    {
        filter: 
        {
            type: Function,
            required: false,
            default: () => (e) => 
            {
                return true
            }
        }
    })
    
    const pathChange = async (e) =>
    {
        let temp = Array.from((await repository.list(path.value.content.slice(1).join('/'))).data)
        temp = temp.filter(e => props.filter(e))
        explorer.value.content = temp
    }

    const enterFolder = async (e) =>
    {
        path.value.content = [...path.value.content, e.detail.folder.name]
        let temp = Array.from((await repository.list(path.value.content.slice(1).join('/'))).data)
        temp = temp.filter(e => props.filter(e))
        explorer.value.content = temp
    }

    const refresh = async () =>
    {
        let temp = Array.from((await repository.list(path.value.content.slice(1).join('/'))).data)
        temp = temp.filter(e => props.filter(e))
        explorer.value.content = temp
    }

    const getSelected = (parametr) => 
    {
        return {path: path.value.content, name: explorer.value.getSelectedItems()[0].name} 
    };

    defineExpose(
    {
        getSelected
    });

    onMounted(async () => 
    {
        explorer.value.types = [{type: 'file', icon: 'F390', isLikeDir: false}, 
        {type: 'dir', icon: 'F3D8', isLikeDir: true}, 
        {type: 'schlib', icon: 'F3D5', isLikeDir: true},
        {type: 'pcblib', icon: 'F3D5', isLikeDir: true},
        {type: 'footprint', icon: 'F6E7', isLikeDir: false}, 
        {type: 'symbol', icon: 'F6E2', isLikeDir: false}]
        
        path.value.content = [(await repository.name()).data]
        let temp = Array.from((await repository.list('/')).data)
        temp = temp.filter(e => props.filter(e))
        explorer.value.content = temp
    })
</script>

<template>
    <onyks-path ref="path" @path-change="pathChange"></onyks-path>
    <onyks-file-explorer ref="explorer" @enter-folder="enterFolder"></onyks-file-explorer>
    <onyks-container type="group" padding="" justify="end">
        <onyks-button @click="refresh" background="green">Refresh</onyks-button>
    </onyks-container>
</template>

<style lang="css" scoped>
    onyks-file-explorer
    {
        width: 100%;
        height: 200px;
    }

    onyks-button
    {
        width: 120px;
    }
</style>