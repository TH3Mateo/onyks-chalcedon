import { defineStore } from 'pinia'
import { ref } from 'vue'
import { LazyStore } from '@tauri-apps/plugin-store';


export const useUserStore = defineStore('user', () => 
{
    const store = new LazyStore('settings.json');

    const loadSettings = async () =>
    {
        login.value = await store.get('login') ?? ''
        password.value = await store.get('password') ?? ''
        webManagerAddress.value = await store.get('webManagerAddress') ?? ''
        // Merged over the defaults, so settings saved by an older version still get
        // the keys added since then.
        repository.value = { ...defaultRepository(), ...(await store.get('repository') ?? {}) }
        database.value = { ...defaultDatabase(), ...(await store.get('database') ?? {}) }
    }

    const saveSettings = async () =>
    {
        await store.set('login', login.value)
        await store.set('password', password.value)
        await store.set('webManagerAddress', webManagerAddress.value)
        await store.set('repository', repository.value)
        await store.set('database', database.value)
        await store.save()
    }

    const init = async () =>
    {
        isReady.value = false
        await loadSettings()
        isReady.value = true
    }
    
    const isReady = ref(false)
    const login = ref('')
    const password = ref('')
    const webManagerAddress = ref('')

    const defaultRepository = () => (
    {
        address: '',
        autoupdate: false,
        autoupdateInterval: 0,
        lastLocalCheckUpdate: 0,
        path: ''
    })

    // The server address comes from webManagerAddress; dblibPath is the local Altium
    // .DbLib file kept up to date with the database.
    const defaultDatabase = () => (
    {
        autoupdate: false,
        autoupdateInterval: 5,
        lastLocalCheckUpdate: 0,
        dblibPath: ''
    })

    const repository = ref(defaultRepository())
    const database = ref(defaultDatabase())
    
    return {
        login,
        password,
        repository,
        database,
        webManagerAddress,
        init,
        loadSettings,
        saveSettings,
        isReady
    }
})