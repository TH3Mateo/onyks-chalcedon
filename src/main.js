import { createApp } from "vue";
import App from "./App.vue";
import router from './router/index.js'
import { createPinia } from 'pinia'
import 'onyks-web-ui'
import 'onyks-web-ui/onyks-web-ui.css'

const pinia = createPinia()
const app = createApp(App)

app.use(router)
app.use(pinia)
app.mount('#app')