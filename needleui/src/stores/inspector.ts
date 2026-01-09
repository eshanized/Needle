// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api'
import type { TunnelRequest } from '@/types'

export const useInspectorStore = defineStore('inspector', () => {
    const requests = ref<TunnelRequest[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchRequests(tunnelId: string, limit: number = 50) {
        loading.value = true
        error.value = null
        try {
            const res = await api.get(`/api/tunnels/${tunnelId}/requests`, { params: { limit } })
            requests.value = res.data.requests || []
        } catch (e: any) {
            error.value = e.response?.data?.error || 'failed to load requests'
        } finally {
            loading.value = false
        }
    }

    function clear() {
        requests.value = []
    }

    return { requests, loading, error, fetchRequests, clear }
})
