// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api'
import type { User, AuthResponse } from '@/types'

export const useAuthStore = defineStore('auth', () => {
    const user = ref<User | null>(null)
    const token = ref<string | null>(localStorage.getItem('needle_token'))
    const loading = ref(false)
    const error = ref<string | null>(null)

    const isAuthenticated = computed(() => !!token.value)

    async function login(email: string, password: string) {
        loading.value = true
        error.value = null
        try {
            const res = await api.post<AuthResponse>('/api/auth/login', { email, password })
            token.value = res.data.token
            user.value = res.data.user
            localStorage.setItem('needle_token', res.data.token)
        } catch (e: any) {
            error.value = e.response?.data?.error || 'login failed'
            throw e
        } finally {
            loading.value = false
        }
    }

    async function register(email: string, username: string, password: string) {
        loading.value = true
        error.value = null
        try {
            const res = await api.post<AuthResponse>('/api/auth/register', { email, username, password })
            token.value = res.data.token
            user.value = res.data.user
            localStorage.setItem('needle_token', res.data.token)
        } catch (e: any) {
            error.value = e.response?.data?.error || 'registration failed'
            throw e
        } finally {
            loading.value = false
        }
    }

    function logout() {
        token.value = null
        user.value = null
        localStorage.removeItem('needle_token')
    }

    return { user, token, loading, error, isAuthenticated, login, register, logout }
})
