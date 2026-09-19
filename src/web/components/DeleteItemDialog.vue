<script setup>
    import { ref } from 'vue'

    const props = defineProps(['action', 'formater', 'subject', 'processor'])
    const emit = defineEmits(['success'])
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    
    const dialog = ref(null)
    const error = ref('')
    const items = ref([])
    const currentCounter = ref(0)

    const open = async (a) => 
    {
        items.value = a
        currentCounter.value = 0

        error.value = ''
        dialog.value.open = true
    }

    const close = () =>
    {
        dialog.value.open = false
    }

    const action = async () =>
    {
        let errors = []
        let result = null
        let text = ''
        for(let i=0; i<items.value.length; i++)
        {
            result = await props.action(props.processor(items.value[i]))
            await sleep(50)
            currentCounter.value += 1
            if(result.status != 200)
            {
                text = result.response.data.detail
                errors.push(props.formater(items.value[i]))
            }
            await sleep(50)
        }
        error.value = 'Error during deleting: ' + errors.toString().replaceAll(',', ', ') + " - " + text
        if(errors.length == 0)
        {
            dialog.value.open = false
            emit('success')
        }
    }

    defineExpose({
        open,
        close
    });
</script>

<template>
    <onyks-dialog :title="`Deleting`" corner-close bottom-buttons modal ref="dialog" @dialog-close="emit('success')">
        <onyks-container type="stack" padding="" gap="l" style="overflow-y: hidden;">
            <slot name="top"></slot>
            <onyks-text>You are deleting {{ items.length }} {{ props.subject }}:</onyks-text>
            <onyks-loading-bar striped animated .max="`${items.length}`" :current-state="currentCounter" color="blue" size="m"></onyks-loading-bar>
            <onyks-container type="stack" gap="l" style="overflow-y: scroll">
                <onyks-text v-for="item in items" size="s">{{ props.formater(item) }}</onyks-text>
            </onyks-container>
            <onyks-text :class="error === '' ? 'error-invisible' : ''">{{ error }}</onyks-text>
        </onyks-container>
        <onyks-button background="green" slot="footer" @click="action" size="m">OK</onyks-button>
        <onyks-button background="red" slot="footer" @click="() => {dialog.open = false; emit('success')}">Close</onyks-button>
    </onyks-dialog>
</template>

<style lang="css" scoped>
    onyks-textfield
    {
        width: 100%;
    }

    .error-invisible
    {
        visibility: hidden;
    }

    onyks-dialog
    {
        position: fixed;
    }

    onyks-dialog
    {
        position: fixed;
    }

    onyks-dialog::part(container)
    {
        max-height: 70vh;
    }
</style>