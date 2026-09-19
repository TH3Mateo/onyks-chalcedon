<script setup lang="js">
    import WarningAlert from '@web/components/WarningAlert.vue';
    import { onMounted, ref } from 'vue';
    import { element, repository, table, manufacturer, supplier } from '@web/utils/api';
    import { dateUTCtoDestination } from '@web/utils/tools';

    const lastAddedElement = ref({
        partName: 'Undefined',
        manufacturer: 'Undefined',
        table: 'Undefined',
        createdAt: 'Undefined'
    })
    
    const repositoryStatistics = ref({
        symbols: 'Undefined',
        footprints: 'Undefined',
        schLibFiles: 'Undefined',
        pcbLibFiles: 'Undefined'
    })

    const elements = ref('Undefined')
    const tables = ref('Undefined')
    const manufacturers = ref('Undefined')
    const suppliers = ref('Undefined')
    const numberOfManufacturers = ref({})
    const numberOfTables = ref({})

    const updater = async () =>
    {
        let data = await element.lastAdded()
        if(data.status == 200)
        {
            data.data.createdAt = dateUTCtoDestination(data.data.createdAt)
            lastAddedElement.value = data.data
        }

        repositoryStatistics.value = (await repository.statistics()).data
        elements.value = (await element.number()).data
        tables.value = (await table.number()).data
        manufacturers.value = (await manufacturer.number()).data
        suppliers.value = (await supplier.number()).data
        numberOfTables.value = (await table.numbers()).data
        numberOfManufacturers.value = (await manufacturer.numbers()).data
    }
    let updaterInverval = null

    onMounted(async () =>
    {
        updater()
        updaterInverval = setInterval(updater, 5000)
    })
</script>

<template>
    <onyks-container gap="l" padding="l">

        <onyks-header>Dashboard</onyks-header>

        <WarningAlert></WarningAlert>

        <onyks-header level="3">Overview</onyks-header>

        <onyks-container gap="l" type="grid" cols="2">

                <onyks-card title="Last added element" size="l">
                    <onyks-container gap="m" padding="">
                        <onyks-container type="group" align="center" padding="" cols="2">
                            <onyks-header level="6">Part name:</onyks-header>
                            <onyks-text>{{ lastAddedElement.partName}}</onyks-text>
                        </onyks-container>
                        <onyks-container type="group" align="center" padding="" cols="2">
                            <onyks-header level="6">Manufacturer:</onyks-header>
                            <onyks-text>{{ lastAddedElement.manufacturer || 'Undefined' }}</onyks-text>
                        </onyks-container>
                        <onyks-container type="group" align="center" padding="" cols="2">
                            <onyks-header level="6">Table:</onyks-header>
                            <onyks-text>{{ lastAddedElement.table || 'Undefined'}}</onyks-text>
                        </onyks-container>
                        <onyks-container type="group" align="center" padding="" cols="2">
                            <onyks-header level="6">Created at:</onyks-header>
                            <onyks-text>{{ lastAddedElement.createdAt || 'Undefined'}}</onyks-text>
                        </onyks-container>
                    </onyks-container>
                </onyks-card>


            <onyks-grid cols="4">
                <onyks-card title="Elements" span="2" size="l">
                    <onyks-header level="3">{{ elements }}</onyks-header>
                </onyks-card>

                <onyks-card title="Tables" span="2" size="l">
                    <onyks-header level="3">{{ tables }}</onyks-header>
                </onyks-card>

                <onyks-card title="Manufacturers" span="2" size="l">
                    <onyks-header level="3">{{ manufacturers }}</onyks-header>
                </onyks-card>

                <onyks-card title="Suppliers" span="2" size="l">
                    <onyks-header level="3">{{ suppliers }}</onyks-header>
                </onyks-card>
            </onyks-grid>
        </onyks-container>

            <onyks-card title="Repository statistics" span="4" size="l">
                <onyks-container gap="m" padding="">
                    <onyks-text size="s">Preliminary data — will reflect the real repository once the SVN hooks update system is finished.</onyks-text>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">Symbols:</onyks-header>
                        <onyks-text>{{ repositoryStatistics.symbols }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">Footprints:</onyks-header>
                        <onyks-text>{{ repositoryStatistics.footprints }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">*.SchLib files:</onyks-header>
                        <onyks-text>{{ repositoryStatistics.schLibFiles }}</onyks-text>
                    </onyks-container>
                    <onyks-container type="group" align="center" padding="" cols="2">
                        <onyks-header level="6">.PcbLib files:</onyks-header>
                        <onyks-text>{{repositoryStatistics.pcbLibFiles}}</onyks-text>
                    </onyks-container>
                </onyks-container>
            </onyks-card>

        <onyks-header level="3">Tables</onyks-header>

        <onyks-grid cols="6" v-if="Object.keys(numberOfTables).length > 0"  gap="l">
            <onyks-card v-for="(number, name) in numberOfTables" :key="name" :title="name" span="2" size="l">
                <onyks-header level="2">{{ number }}</onyks-header>
            </onyks-card>
        </onyks-grid>

        <onyks-text v-else>There is no tables to show.</onyks-text>

        <onyks-header level="3">Manufacturers</onyks-header>
        
        <onyks-grid cols="6" v-if="Object.keys(numberOfManufacturers).length > 0"  gap="l">
            <onyks-card v-for="(number, name) in numberOfManufacturers" :title="name" span="2" size="l">
                <onyks-header level="2">{{ number }}</onyks-header>
            </onyks-card>
        </onyks-grid>

        <onyks-text v-else>There is no manufacturers to show.</onyks-text>

    </onyks-container>
</template>

<style scoped>
    onyks-container > onyks-header
    {
        width: 120px;
        text-align: left;
    }
</style>