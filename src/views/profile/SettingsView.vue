<script setup>
    import ProfilePage from '../../components/ProfilePage.vue';
    import { useUserStore } from '../../stores/user.js';
    import { ref } from 'vue';
    import { open, save as saveDialog } from '@tauri-apps/plugin-dialog';
    import { useDbLibSync } from '../../composables/useDbLibSync.js';
    import { useRepositorySync } from '../../composables/useRepositorySync.js';
    import { useTailscale } from '../../composables/useTailscale.js';
    import { useServerDiscovery, portOf, replaceHost, defaultRepositoryAddress } from '../../composables/useServerDiscovery.js';

    const userStore = useUserStore()
    const visibleSettings = ref(JSON.parse(JSON.stringify(userStore.$state)))

    const reset = async () =>
    {
        await userStore.loadSettings()
        visibleSettings.value = JSON.parse(JSON.stringify(userStore.$state))
        showNotice('success', 'Changes discarded.')
    }

    // Feedback for Save / Reset: a short-lived alert above the buttons.
    const notice = ref({ type: 'success', text: '' })
    let noticeTimer = null

    const showNotice = (type, text) =>
    {
        notice.value = { type, text }
        clearTimeout(noticeTimer)
        noticeTimer = setTimeout(() => { notice.value.text = '' }, 3000)
    }

    const save = async () =>
    {
        try
        {
            userStore.$patch(JSON.parse(JSON.stringify(visibleSettings.value)))
            await userStore.saveSettings()
            showNotice('success', 'Settings saved.')
        }
        catch (error)
        {
            showNotice('error', `Could not save the settings: ${error}`)
        }
    }

    const chooseRepositoryPath = async () =>
    {
        const file = await open({
            multiple: false,
            directory: true,
        });

        visibleSettings.value.repository.path = file
    }

    const { status: dblibStatus, syncDbLib } = useDbLibSync()
    const { status: repositoryStatus } = useRepositorySync()

    const chooseDbLibPath = async () =>
    {
        const file = await saveDialog({
            defaultPath: visibleSettings.value.database.dblibPath || 'onyks_bloodstone.DbLib',
            filters: [{ name: 'Altium Database Library', extensions: ['DbLib'] }]
        });

        if(file)
        {
            visibleSettings.value.database.dblibPath = file
        }
    }

    const chooseKicadDblPath = async () =>
    {
        const file = await saveDialog({
            defaultPath: visibleSettings.value.database.kicadDblPath || 'onyks_bloodstone.kicad_dbl',
            filters: [{ name: 'KiCad Database Library', extensions: ['kicad_dbl'] }]
        });

        if(file)
        {
            visibleSettings.value.database.kicadDblPath = file
        }
    }

    const updateDbLibNow = async () =>
    {
        await save()
        await syncDbLib()
    }

    const { status: tailscaleStatus, connectTailscale } = useTailscale()

    const chooseTailscaleKeyPath = async () =>
    {
        const file = await open({
            multiple: false,
            directory: false,
            filters: [{ name: 'Text file', extensions: ['txt'] }]
        });

        if(file)
        {
            visibleSettings.value.tailscale.authKeyPath = file
        }
    }

    const connectTailscaleNow = async () =>
    {
        await save()
        await connectTailscale()
    }

    const { status: discoveryStatus, discoverServers } = useServerDiscovery()

    const findServer = () => discoverServers(portOf(visibleSettings.value.webManagerAddress))

    // The web manager lives on the same machine as the SVN server: an empty repository
    // address is filled in (http://<ip>:8111/svn/part_library), an existing one follows
    // the new IP.
    const useServer = (server) =>
    {
        visibleSettings.value.webManagerAddress = server.address
        if (visibleSettings.value.repository.address)
        {
            visibleSettings.value.repository.address = replaceHost(visibleSettings.value.repository.address, server.ip)
        }
        else
        {
            visibleSettings.value.repository.address = defaultRepositoryAddress(server.ip)
        }
    }

    const formatTime = (date) => date ? date.toLocaleString() : 'never'
</script>

<template>
    <ProfilePage title="Settings">

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Login</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.login" placeholder="e.g admin"></onyks-textfield>
            <onyks-text size="m" >Password</onyks-text>
            <onyks-textfield size="m" type="password" v-model="visibleSettings.password" placeholder="e.g admin"></onyks-textfield>
        </onyks-container>

        <onyks-header level="4">Web Manager</onyks-header>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Address</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.webManagerAddress" placeholder="e.g http://192.168.1.10:8113"></onyks-textfield>
        </onyks-container>

        <onyks-text size="s">
            "Find server" searches the local network (also over VPN / Tailscale) for the ONYKS server,
            so you do not have to look up its IP address. Using a found server also fills in the
            repository address below (http://&lt;ip&gt;:8111/svn/part_library) or, if it is already set,
            updates its host.
        </onyks-text>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s" v-if="discoveryStatus.running">Searching the network...</onyks-text>
            <onyks-text size="s" v-else-if="discoveryStatus.searched && discoveryStatus.servers.length == 0">No server found.</onyks-text>
            <onyks-button background="blue" @click="findServer" :disabled="discoveryStatus.running">Find server</onyks-button>
        </onyks-container>

        <onyks-container v-for="server in discoveryStatus.servers" :key="server.ip" gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">{{ server.address }} {{ server.verified ? '· ONYKS server' : '· unknown service' }}</onyks-text>
            <onyks-button background="green" @click="useServer(server)">Use</onyks-button>
        </onyks-container>

        <onyks-alert v-if="discoveryStatus.error" type="error">{{ discoveryStatus.error }}</onyks-alert>

        <onyks-header level="4">Database</onyks-header>

        <onyks-text size="s">
            The program keeps the library file of your CAD tool up to date - the Altium .DbLib or the KiCad
            .kicad_dbl, depending on the account (the server tells which one it is). With autoupdate on it
            checks the database periodically and adds every new category table and every new supplier column.
            Neither file contains a password: the connection and the login come from the PostgreSQL ODBC driver.
            In the Altium file the connection settings and anything else set in Altium are kept; if it does not
            exist yet, it is created with the ODBC data source "bloodstone". The KiCad file is generated by the server.
            Keep the file outside the repository folder, otherwise Push would commit it.
        </onyks-text>

        <onyks-text size="s">
            CAD tool of the account: {{ dblibStatus.tool ?? 'not detected yet (it is checked with Update now and on every autoupdate)' }}
        </onyks-text>

        <onyks-container v-if="dblibStatus.tool !== 'kicad'" type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">DbLib File Path (Altium)</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.database.dblibPath" placeholder="e.g C:/User/onyks_bloodstone.DbLib" disabled></onyks-textfield>
        </onyks-container>

        <onyks-container v-if="dblibStatus.tool !== 'kicad'" gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="yellow" @click="chooseDbLibPath">Select</onyks-button>
        </onyks-container>

        <onyks-container v-if="dblibStatus.tool !== 'altium'" type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">.kicad_dbl File Path (KiCad)</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.database.kicadDblPath" placeholder="e.g C:/User/onyks_bloodstone.kicad_dbl" disabled></onyks-textfield>
        </onyks-container>

        <onyks-container v-if="dblibStatus.tool !== 'altium'" gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="yellow" @click="chooseKicadDblPath">Select</onyks-button>
        </onyks-container>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Autoupdate</onyks-text>
            <onyks-checkbox size="l" :checked="visibleSettings.database.autoupdate" @click="() => {visibleSettings.database.autoupdate = !visibleSettings.database.autoupdate}"></onyks-checkbox>
        </onyks-container>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Autoupdate Interval (in seconds)</onyks-text>
            <onyks-textfield placeholder="e.g 5" size="m" type="number" v-model="visibleSettings.database.autoupdateInterval" :disabled="!visibleSettings.database.autoupdate"></onyks-textfield>
        </onyks-container>
        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">Min. value of the interval is 5 seconds.</onyks-text>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">
                Last check: {{ formatTime(dblibStatus.lastCheck) }} · Last change: {{ formatTime(dblibStatus.lastChange) }}
            </onyks-text>
            <onyks-button background="blue" @click="updateDbLibNow" :disabled="dblibStatus.running">Update now</onyks-button>
        </onyks-container>

        <onyks-alert v-if="dblibStatus.error" type="error">{{ dblibStatus.error }}</onyks-alert>

        <onyks-header level="4">Repository</onyks-header>

        <onyks-text size="s">
            Two independent, optional ways to keep the SVN working copy in sync without using the
            Push/Pull buttons by hand. Autoupdate (Pull) periodically downloads changes from the
            server. Auto-push watches the folder and all its subfolders and commits automatically
            the moment any file in it changes. Both can be on at once; only one SVN operation ever
            runs at a time.
        </onyks-text>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Address</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.repository.address" placeholder="e.g https://google.com"></onyks-textfield>
            <onyks-text size="m">Folder Path</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.repository.path" placeholder="e.g C:/User/repository" disabled></onyks-textfield>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="yellow" @click="chooseRepositoryPath">Select</onyks-button>
        </onyks-container>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Autoupdate (Pull)</onyks-text>
            <onyks-checkbox size="l" :checked="visibleSettings.repository.autoupdate" @click="() => {visibleSettings.repository.autoupdate = !visibleSettings.repository.autoupdate}"></onyks-checkbox>
        </onyks-container>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Autoupdate Interval (in seconds)</onyks-text>
            <onyks-textfield size="m" placeholder="e.g 5" type="number" v-model="visibleSettings.repository.autoupdateInterval" :disabled="!visibleSettings.repository.autoupdate"></onyks-textfield>
        </onyks-container>
        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">Min. value of the interval is 5 seconds.</onyks-text>
        </onyks-container>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Auto-push on file change</onyks-text>
            <onyks-checkbox size="l" :checked="visibleSettings.repository.autoupdatePush" @click="() => {visibleSettings.repository.autoupdatePush = !visibleSettings.repository.autoupdatePush}"></onyks-checkbox>
        </onyks-container>
        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">Commits automatically a few seconds after any change under the folder path above.</onyks-text>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">
                Last pull: {{ formatTime(repositoryStatus.lastPullCheck) }} · Last push: {{ formatTime(repositoryStatus.lastPush) }}
            </onyks-text>
        </onyks-container>

        <onyks-alert v-if="repositoryStatus.error" type="error">{{ repositoryStatus.error }}</onyks-alert>

        <onyks-header level="4">Tailscale</onyks-header>

        <onyks-text size="s">
            With this option on, the program starts the Tailscale client when it starts and signs in
            to your Tailscale network with the personal auth key from the text file selected below.
            Only the path to the file is stored in the settings, never the key itself.
        </onyks-text>

        <onyks-container type="grid" cols="2" gap="l" padding="">
            <onyks-text size="m">Connect to Tailscale on startup</onyks-text>
            <onyks-checkbox size="l" :checked="visibleSettings.tailscale.autostart" @click="() => {visibleSettings.tailscale.autostart = !visibleSettings.tailscale.autostart}"></onyks-checkbox>
            <onyks-text size="m">Auth Key File (.txt)</onyks-text>
            <onyks-textfield size="m" v-model="visibleSettings.tailscale.authKeyPath" placeholder="e.g C:/User/tailscale_key.txt" disabled></onyks-textfield>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="yellow" @click="chooseTailscaleKeyPath">Select</onyks-button>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-text size="s">
                {{ tailscaleStatus.lastConnect ? `Last connection: ${formatTime(tailscaleStatus.lastConnect)} · ${tailscaleStatus.message}` : 'Not connected in this session.' }}
            </onyks-text>
            <onyks-button background="blue" @click="connectTailscaleNow" :disabled="tailscaleStatus.running || !visibleSettings.tailscale.authKeyPath">Connect now</onyks-button>
        </onyks-container>

        <onyks-alert v-if="tailscaleStatus.error" type="error">{{ tailscaleStatus.error }}</onyks-alert>

        <onyks-alert v-if="notice.text" :type="notice.type">{{ notice.text }}</onyks-alert>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="">
            <onyks-button background="green" @click="save">Save</onyks-button>
            <onyks-button background="red" @click="reset">Reset</onyks-button>
        </onyks-container>

        <onyks-container gap="m" type="group" align="center" justify="end" padding="s">
        </onyks-container>
    </ProfilePage>
</template>

<style scoped>
    onyks-text
    {
        align-self: center;
    }

    onyks-textfield
    {
        width: 100%;
    }

    onyks-button
    {
        width: 120px;
    }

    onyks-checkbox
    {
        width: fit-content;
    }
</style>