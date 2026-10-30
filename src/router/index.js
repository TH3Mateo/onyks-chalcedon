import { createRouter, createWebHistory } from 'vue-router'
import StartView from '../views/StartView.vue'
import ProfileView from '../views/ProfileView.vue'
import RepositoryView from '../views/profile/RepositoryView.vue'
import SettingsView from '../views/profile/SettingsView.vue'
import ConvertView from '../views/profile/ConvertView.vue'
import webRoutes from '../web/routes.js'

const routes = [
  {
    path: '/start',
    name: 'Start',
    component: StartView
  },
  {
    path: '/',
    redirect: '/profile/repository'
  },
  {
    path: '/profile',
    name: 'Profile',
    component: ProfileView,
    children: 
    [
      {
        path: 'repository',
        name: 'Repository',
        component: RepositoryView
      },
      {
        path: 'settings',
        name: 'Settings',
        component: SettingsView
      },
      {
        path: 'convert',
        name: 'Convert',
        component: ConvertView
      },
      webRoutes,
    ]
  }
]

const router = createRouter(
{
  history: createWebHistory(),
  routes
})

export default router