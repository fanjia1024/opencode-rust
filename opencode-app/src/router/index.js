import { createRouter, createWebHistory } from 'vue-router'
import Empty from '../views/Empty.vue'
import Session from '../views/Session.vue'
import Settings from '../views/Settings.vue'
import Help from '../views/Help.vue'

const routes = [
  { path: '/', name: 'Empty', component: Empty },
  { path: '/session/:id', name: 'Session', component: Session },
  { path: '/settings', name: 'Settings', component: Settings },
  { path: '/help', name: 'Help', component: Help },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
