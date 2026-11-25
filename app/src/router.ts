import { createRouter, createWebHashHistory } from 'vue-router'
import LibraryView from './views/LibraryView.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'library', component: LibraryView },
    { path: '/community', name: 'community', component: () => import('./views/CommunityView.vue') },
    { path: '/my-wallpapers', name: 'my-wallpapers', component: () => import('./views/MyWallpapersView.vue') },
    { path: '/settings', name: 'settings', component: () => import('./views/SettingsView.vue') },
  ],
})

export default router
