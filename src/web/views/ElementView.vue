<script setup lang="js">
    import WarningAlert from '@web/components/WarningAlert.vue';
    import ElementForm from '@web/components/ElementForm.vue';
    import { ElementModel } from '@web/utils/db';
    import { ref, toRaw } from 'vue';
    import { element } from '@web/utils/api';
    import { useRouter } from 'vue-router';
    import { useRoute } from 'vue-router';
    import { dateUTCtoDestination } from '@web/utils/tools';
    import BasicButtonsPanel from '@web/components/BasicButtonsPanel.vue';
    import DeleteItemDialog from '@web/components/DeleteItemDialog.vue';
    import { LabelsDoc } from '@web/utils/tools';
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

    const props = defineProps(['type'])
    const model = ref(new ElementModel())
    const dialogs = ref({create: null, addError: null, delete: null, editError: null, edit: null, duplicate: null, duplicateError: null})
    const router = useRouter()
    const route = useRoute()
    const error = ref('')
    
    const file = ref(null)

    const fillData = () =>
    {
        element.get(route.params.uuid).then((e) =>
        {
            if(e.status == 200)
            {
                e.data.createdAt = dateUTCtoDestination(e.data.createdAt)
                Object.assign(model.value, e.data);
            }
        })
    }

    switch(props.type)
    {
        case 'add':
            break;
        case 'details':
            fillData()
            break;
        case 'edit':
            fillData()
            break;
        case 'duplicate':
            fillData()
            break;
    }

    const action = async (mode) =>
    {
        let data = null
        switch(mode)
        {
            case 'create':
                dialogs.value.create.open = true
                data = await element.create(model.value, file.value)
                if(data.status == 200)
                {
                    setTimeout(() => 
                    {
                        dialogs.value.create.open = false
                        router.push(`/profile/web/element/details/${data.data.uuid}`)
                    }, 1000)
                }
                else
                {
                    error.value = data.response.data.detail
                    await sleep(1000)
                    dialogs.value.create.open = false
                    dialogs.value.addError.open = true
                }
                break;
            case 'edit':
                // console.log(file.value)
                // console.log(model.value)
                dialogs.value.edit.open = true
                const updateData = structuredClone(toRaw(model.value))
                data = await element.edit(model.value.uuid, updateData, file.value)
                if(data.status == 200)
                {
                    setTimeout(() => 
                    {
                        dialogs.value.edit.open = false
                        router.push(`/profile/web/element/details/${data.data.uuid}`)
                    }, 1000)
                }
                else
                {
                    error.value = data.response.data.detail
                    await sleep(1000)
                    dialogs.value.edit.open = false
                    dialogs.value.editError.open = true
                }
                break;
            case 'duplicate':
                dialogs.value.duplicate.open = true
                const duplicateData = structuredClone(toRaw(model.value))
                duplicateData.uuid = null
                if(!model.value.datasheet && file.value == null)
                {
                    duplicateData.isDatasheetSupposedToChange = 1
                }
                data = await element.duplicate(model.value.uuid, duplicateData, file.value)
                if(data.status == 200)
                {
                    setTimeout(() => 
                    {
                        dialogs.value.duplicate.open = false
                        router.push(`/profile/web/element/details/${data.data.uuid}`)
                    }, 1000)
                }
                else
                {
                    duplicateData.isDatasheetSupposedToChange = 2
                    error.value = data.response.data.detail
                    await sleep(1000)
                    dialogs.value.duplicate.open = false
                    dialogs.value.duplicateError.open = true

                }
                break;
        }
    }

    const label = async () =>
    {
        const doc = new LabelsDoc(1)
        await doc.init()

        await doc.drawData(0, model.value)
        await doc.drawQR(0, model.value.uuid)

        await doc.drawBorders()
        await doc.finish()
    }
</script>

<template>
  <onyks-container gap="l" padding="l">

        <onyks-header v-if="props.type == 'add'">Create an element</onyks-header>
        <onyks-header v-else-if="props.type == 'duplicate'">Duplicate an element</onyks-header>
        <onyks-header v-else-if="props.type == 'details'">Element's details</onyks-header>
        <onyks-header v-if="props.type == 'edit'">Edit an element</onyks-header>

        <WarningAlert></WarningAlert>

        <onyks-container align="start" padding="" gap="l">
            <onyks-button background="red" @click="router.back()">Return</onyks-button>
        </onyks-container>

        <ElementForm v-model="model" :type="props.type" v-model:file="file"></ElementForm>
       
        <onyks-dialog modal no-title :ref="(el) => { dialogs.create = el }">
            <onyks-text>Creating the element...</onyks-text>
        </onyks-dialog>

        <onyks-dialog modal no-title :ref="(el) => { dialogs.edit = el }">
            <onyks-text>Editing the element...</onyks-text>
        </onyks-dialog>

        <onyks-dialog modal no-title :ref="(el) => { dialogs.duplicate = el }">
            <onyks-text>Duplicating the element...</onyks-text>
        </onyks-dialog>

        <onyks-dialog :title="`Error`" modal corner-close :ref="(el) => { if (el && dialogs) dialogs.addError = el }">
            <onyks-text>{{ error }}</onyks-text>
        </onyks-dialog>

        <onyks-dialog :title="`Error`" modal corner-close :ref="(el) => { if (el && dialogs) dialogs.editError = el }">
            <onyks-text>{{ error }}</onyks-text>
        </onyks-dialog>

        <onyks-dialog :title="`Error`" modal corner-close :ref="(el) => { if (el && dialogs) dialogs.duplicateError = el }">
            <onyks-text>{{ error }}</onyks-text>
        </onyks-dialog>

        <BasicButtonsPanel v-if="props.type == 'details'">
            <onyks-button background="blue" @click="() => {router.push(`/profile/web/element/edit/${route.params.uuid}`)}">Edit</onyks-button>
            <onyks-button background="yellow" @click="() => {router.push(`/profile/web/element/duplicate/${route.params.uuid}`)}">Duplicate</onyks-button>
            <onyks-button background="red" @click="() => {dialogs.delete.open([model])}">Delete</onyks-button>
            <onyks-button background="green" @click="model.datasheet? element.openDatasheet(route.params.uuid): null" :disabled="!model.datasheet">Datasheet</onyks-button>
            <onyks-button background="blue" @click="label">Label</onyks-button> 
        </BasicButtonsPanel>

        <onyks-container align="end" padding="" gap="l" v-else-if="props.type == 'edit'">
            <onyks-button background="blue" @click="() => {action('edit')}">Edit</onyks-button>
        </onyks-container>

        <onyks-container align="end" padding="" gap="l" v-else-if="props.type == 'duplicate'">
            <onyks-button background="yellow" @click="() => {action('duplicate')}">Duplicate</onyks-button>
        </onyks-container>

        <onyks-container align="end" padding="" gap="l" v-else>
            <onyks-button background="green" @click="() => {action('create')}">Create</onyks-button>
        </onyks-container>

        <DeleteItemDialog
            subject="element"
            :processor="(item) => item.uuid"
            :ref="(el) => { if (el && dialogs) dialogs.delete = el }"
            :action="element.delete"
            :formater="(item) => item.partName"
            @success="router.push('/profile/web/management')">
            <template v-slot:top>
                <onyks-alert type="warning">This operation cannot be undone.</onyks-alert>
            </template>
        </DeleteItemDialog>

    </onyks-container>

</template>

<style>
    onyks-dialog
    {
        position: fixed;
    }
</style>