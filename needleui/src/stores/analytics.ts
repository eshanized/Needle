// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api'

export interface DailyStats {
    date: string
    total_requests: number
    total_bytes_in: number
    total_bytes_out: number
    avg_latency_ms: number
    error_count: number
    unique_ips: number
}

export interface AnalyticsSummary {
    total_tunnels: number
    requests_7d: number
    bytes_7d: number
}

export const useAnalyticsStore = defineStore('analytics', () => {
    const dailyStats = ref<DailyStats[]>([])
    const summary = ref<AnalyticsSummary | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    async function fetchTunnelStats(tunnelId: string, days: number = 30) {
        loading.value = true
        error.value = null
        try {
            const res = await api.get(`/api/tunnels/${tunnelId}/analytics`, { params: { days } })
            dailyStats.value = res.data.stats || []
        } catch (e: any) {
            error.value = e.response?.data?.error || 'failed to load analytics'
        } finally {
            loading.value = false
        }
    }

    async function fetchSummary() {
        try {
            const res = await api.get('/api/analytics/summary')
            summary.value = res.data.summary
        } catch (e: any) {
            error.value = e.response?.data?.error || 'failed to load summary'
        }
    }

    return { dailyStats, summary, loading, error, fetchTunnelStats, fetchSummary }
})
