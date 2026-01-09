// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/login',
            name: 'login',
            component: () => import('@/views/Login.vue'),
            meta: { public: true },
        },
        {
            path: '/register',
            name: 'register',
            component: () => import('@/views/Register.vue'),
            meta: { public: true },
        },
        {
            path: '/',
            name: 'dashboard',
            component: () => import('@/views/Dashboard.vue'),
        },
        {
            path: '/tunnels',
            name: 'tunnels',
            component: () => import('@/views/Tunnels.vue'),
        },
        {
            path: '/tunnels/:subdomain',
            name: 'tunnel-detail',
            component: () => import('@/views/TunnelDetail.vue'),
        },
        {
            path: '/settings',
            name: 'settings',
            component: () => import('@/views/Settings.vue'),
        },
        {
            path: '/tunnels/:tunnelId/inspector',
            name: 'inspector',
            component: () => import('@/views/Inspector.vue'),
        },
        {
            path: '/tunnels/:tunnelId/analytics',
            name: 'analytics',
            component: () => import('@/views/Analytics.vue'),
        },
    ],
})

router.beforeEach((to) => {
    const token = localStorage.getItem('needle_token')
    if (!to.meta.public && !token) {
        return { name: 'login' }
    }
})

export default router
