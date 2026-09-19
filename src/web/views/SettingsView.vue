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

            <onyks-card title="Download Library Files" size="l">
                <onyks-container gap="m" padding="">
                    <onyks-text size="s">
                        Both files are generated fresh from the current database every time you download them —
                        any new category table is reflected automatically. The username is filled in; add the
                        password after opening the file. Altium and KiCad each read their own set of tables and
                        cannot see the other's.
                    </onyks-text>

                    <onyks-button background="blue" @click="settings.downloadDbLib">Download Altium .DbLib</onyks-button>
                    <onyks-button background="green" @click="settings.downloadKicadDbl">Download KiCad .kicad_dbl</onyks-button>

                    <onyks-text size="s">
                        For KiCad: register each SchLib/PcbLib file in your Symbol/Footprint Library Table with
                        Type = Altium, using the file name (without extension) as the library nickname — this
                        must match the "Symbols"/"Footprints" columns generated in the database views.
                    </onyks-text>
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
