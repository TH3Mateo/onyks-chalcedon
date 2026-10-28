<script setup>
    import { ref, onMounted } from 'vue';
    import { settings } from '@web/utils/api.js';

    const identity = ref({
        svnRepository: '',
        databaseHost: '',
        databasePort: '',
        databaseName: ''
    })

    onMounted(async () =>
    {
        const res = await settings.identity()
        if (res.status == 200)
        {
            identity.value = res.data
        }
    })
</script>

<template>
    <onyks-container gap="l" padding="l">

        <onyks-header>Settings</onyks-header>

        <onyks-container gap="l" type="grid" cols="2" padding="">

            <onyks-card title="Connection Info" size="l">
                <onyks-container gap="m" padding="">
                    <onyks-text size="s">
                        Use these details to configure an ODBC Data Source (required by both Altium and KiCad
                        database libraries). Sign in with the same login and password you use for this
                        application and for SVN — it is one account. Your account only grants read access,
                        and only to the tables belonging to your CAD tool.
                    </onyks-text>

                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">Database host:</onyks-header>
                        <onyks-text>{{ identity.databaseHost || 'Undefined' }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">Database port:</onyks-header>
                        <onyks-text>{{ identity.databasePort || 'Undefined' }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">Database name:</onyks-header>
                        <onyks-text>{{ identity.databaseName || 'Undefined' }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">SVN repository:</onyks-header>
                        <onyks-text>{{ identity.svnRepository || 'Undefined' }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">Username:</onyks-header>
                        <onyks-text>your own account</onyks-text>
                    </onyks-container>
                </onyks-container>
            </onyks-card>

        </onyks-container>

    </onyks-container>
</template>

<style scoped>
    onyks-button
    {
        width: 100%;
    }
</style>
