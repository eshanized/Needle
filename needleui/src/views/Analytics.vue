<!-- Author : Eshan Roy <eshanized@proton.me> -->
<!-- SPDX-License-Identifier: MIT -->

<template>
  <div class="app-layout">
    <Sidebar :user-email="authStore.user?.email" @logout="handleLogout" />

    <main class="main-content">
      <div class="page-header">
        <div>
          <router-link :to="`/tunnels/${tunnelId}`" style="font-size: 13px; color: var(--text-muted);">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align: -2px;">
              <polyline points="15 18 9 12 15 6"/>
            </svg>
            Back to tunnel
          </router-link>
          <h1 class="page-title" style="margin-top: 8px;">Analytics</h1>
        </div>
        <div style="display: flex; gap: 8px;">
          <button v-for="d in [7, 14, 30]" :key="d"
            :class="['btn', days === d ? 'btn-primary' : 'btn-ghost']"
            style="padding: 6px 14px; font-size: 13px;"
            @click="changePeriod(d)">
            {{ d }}d
          </button>
        </div>
      </div>

      <div v-if="analyticsStore.loading" class="loading">
        <div class="spinner"></div>
      </div>

      <template v-else>
        <!-- summary cards -->
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-label">Total Requests</div>
            <div class="stat-value">{{ totalRequests.toLocaleString() }}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Avg Latency</div>
            <div class="stat-value">{{ avgLatency }}ms</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Data Transferred</div>
            <div class="stat-value">{{ formatBytes(totalBytes) }}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">Error Rate</div>
            <div class="stat-value" :style="{ color: errorRate > 5 ? 'var(--accent-red)' : 'var(--accent-green)' }">
              {{ errorRate }}%
            </div>
          </div>
        </div>

        <!-- bar chart (css-only) -->
        <div class="card" style="margin-bottom: 24px;">
          <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 20px; text-transform: uppercase; letter-spacing: 0.05em;">
            Requests per Day
          </h3>
          <div v-if="analyticsStore.dailyStats.length === 0" class="empty-state" style="padding: 30px;">
            <p>No data available for this period.</p>
          </div>
          <div v-else class="chart-container">
            <div v-for="day in chartData" :key="day.date" class="chart-bar-wrapper">
              <div class="chart-bar" :style="{ height: day.height + '%' }" :title="`${day.date}: ${day.requests} requests`">
              </div>
              <span class="chart-label">{{ day.label }}</span>
            </div>
          </div>
        </div>

        <!-- daily breakdown table -->
        <div class="card">
          <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em;">
            Daily Breakdown
          </h3>
          <table style="width: 100%; border-collapse: collapse;">
            <thead>
              <tr style="border-bottom: 1px solid var(--border);">
                <th style="text-align: left; padding: 8px 0; font-size: 12px; color: var(--text-muted);">Date</th>
                <th style="text-align: right; padding: 8px 0; font-size: 12px; color: var(--text-muted);">Requests</th>
                <th style="text-align: right; padding: 8px 0; font-size: 12px; color: var(--text-muted);">Avg Latency</th>
                <th style="text-align: right; padding: 8px 0; font-size: 12px; color: var(--text-muted);">Errors</th>
                <th style="text-align: right; padding: 8px 0; font-size: 12px; color: var(--text-muted);">Unique IPs</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="day in analyticsStore.dailyStats" :key="day.date" style="border-bottom: 1px solid var(--border);">
                <td style="padding: 10px 0; font-size: 13px;">{{ day.date }}</td>
                <td style="padding: 10px 0; font-size: 13px; text-align: right;">{{ day.total_requests.toLocaleString() }}</td>
                <td style="padding: 10px 0; font-size: 13px; text-align: right;">{{ day.avg_latency_ms }}ms</td>
                <td style="padding: 10px 0; font-size: 13px; text-align: right;" :style="{ color: day.error_count > 0 ? 'var(--accent-red)' : 'var(--text-secondary)' }">
                  {{ day.error_count }}
                </td>
                <td style="padding: 10px 0; font-size: 13px; text-align: right;">{{ day.unique_ips }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useAnalyticsStore } from '@/stores/analytics'
import Sidebar from '@/components/Sidebar.vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const analyticsStore = useAnalyticsStore()
const days = ref(30)

const tunnelId = computed(() => route.params.tunnelId as string)

const totalRequests = computed(() =>
  analyticsStore.dailyStats.reduce((sum, d) => sum + d.total_requests, 0)
)

const avgLatency = computed(() => {
  const stats = analyticsStore.dailyStats
  if (stats.length === 0) return 0
  const total = stats.reduce((sum, d) => sum + d.avg_latency_ms, 0)
  return Math.round(total / stats.length)
})

const totalBytes = computed(() =>
  analyticsStore.dailyStats.reduce((sum, d) => sum + d.total_bytes_in + d.total_bytes_out, 0)
)

const errorRate = computed(() => {
  if (totalRequests.value === 0) return 0
  const errors = analyticsStore.dailyStats.reduce((sum, d) => sum + d.error_count, 0)
  return Math.round((errors / totalRequests.value) * 1000) / 10
})

const chartData = computed(() => {
  const stats = [...analyticsStore.dailyStats].reverse()
  const max = Math.max(...stats.map(d => d.total_requests), 1)
  return stats.map(d => ({
    date: d.date,
    requests: d.total_requests,
    height: (d.total_requests / max) * 100,
    label: d.date.slice(5), // MM-DD
  }))
})

onMounted(() => {
  analyticsStore.fetchTunnelStats(tunnelId.value, days.value)
})

function changePeriod(d: number) {
  days.value = d
  analyticsStore.fetchTunnelStats(tunnelId.value, d)
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.chart-container {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  height: 160px;
  padding: 0 4px;
}
.chart-bar-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
  justify-content: flex-end;
}
.chart-bar {
  width: 100%;
  min-height: 2px;
  background: var(--accent-cyan);
  border-radius: 2px 2px 0 0;
  transition: height 0.3s ease;
  opacity: 0.8;
}
.chart-bar:hover {
  opacity: 1;
  background: var(--accent-blue);
}
.chart-label {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 6px;
  white-space: nowrap;
}
</style>
