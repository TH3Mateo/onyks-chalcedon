import WebManagerLayout from './WebManagerLayout.vue'
import DashboardView from '@web/views/DashboardView.vue'
import ManagementView from '@web/views/ManagementView.vue'
import RepositoryView from '@web/views/RepositoryView.vue'
import SettingsView from '@web/views/SettingsView.vue'
import ElementView from '@web/views/ElementView.vue'
import ErrorView from '@web/views/ErrorView.vue'

// The Bloodstone web GUI, ported as-is and mounted under /profile/web so that the
// strip menu of ProfileView stays visible next to it.
export default {
  path: 'web',
  component: WebManagerLayout,
  children:
  [
    { path: '', redirect: '/profile/web/dashboard' },
    { path: 'dashboard', name: 'WebDashboard', component: DashboardView },
    { path: 'management', component: ManagementView },
    { path: 'repository', component: RepositoryView },
    { path: 'settings', component: SettingsView },
    { path: 'element/add', component: ElementView, props: { type: 'add' } },
    { path: 'element/details/:uuid', component: ElementView, props: { type: 'details' } },
    { path: 'element/edit/:uuid', component: ElementView, props: { type: 'edit' } },
    { path: 'element/duplicate/:uuid', component: ElementView, props: { type: 'duplicate' } },
    { path: 'error', component: ErrorView },
  ]
}
