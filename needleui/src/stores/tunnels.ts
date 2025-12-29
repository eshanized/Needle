// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api'
import type { Tunnel, CreateTunnelRequest } from '@/types'

export const useTunnelsStore = defineStore('tunnels', () => {
    const tunnels = ref<Tunnel[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    const activeTunnels = computed(() => tunnels.value.filter((t) => t.is_active))
    const activeCount = computed(() => activeTunnels.value.length)

    async function fetchTunnels() {
        loading.value = true
        error.value = null
        try {
            const res = await api.get('/api/tunnels')
            tunnels.value = res.data.tunnels || []
        } catch (e: any) {
            error.value = e.response?.data?.error || 'failed to load tunnels'
        } finally {
            loading.value = false
        }
    }

    async function createTunnel(data: CreateTunnelRequest): Promise<Tunnel | null> {
        try {
            const res = await api.post('/api/tunnels', data)
            await fetchTunnels()
            return res.data
        } catch (e: any) {
            error.value = e.response?.data?.error || 'failed to create tunnel'
            return null
        }
    }

    async function deleteTunnel(subdomain: string) {
        try {
            await api.delete(`/api/tunnels/${subdomain}`)
            tunnels.value = tunnels.value.filter((t) => t.subdomain !== subdomain)
        } catch (e: any) {
            error.value = e.response?.data?.error || 'failed to delete tunnel'
        }
    }

    return { tunnels, loading, error, activeTunnels, activeCount, fetchTunnels, createTunnel, deleteTunnel }
})
