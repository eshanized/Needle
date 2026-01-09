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
          <h1 class="page-title" style="margin-top: 8px;">Traffic Inspector</h1>
        </div>
        <button class="btn btn-ghost" @click="refresh">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/>
          </svg>
          Refresh
        </button>
      </div>

      <div v-if="inspectorStore.loading" class="loading">
        <div class="spinner"></div>
      </div>

      <div v-else-if="inspectorStore.requests.length === 0" class="empty-state">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/><path d="M9 10h.01M15 10h.01M8 15s1.5 2 4 2 4-2 4-2"/>
        </svg>
        <p>No requests recorded yet. Traffic will appear here as it flows through the tunnel.</p>
      </div>

      <div v-else class="tunnel-list">
        <div v-for="req in inspectorStore.requests" :key="req.id" class="request-card card" @click="selectedRequest = selectedRequest?.id === req.id ? null : req">
          <div style="display: flex; align-items: center; gap: 12px; flex: 1;">
            <span :class="['badge', methodBadge(req.method)]">{{ req.method }}</span>
            <span class="detail-value" style="flex: 1;">{{ req.path }}</span>
            <span :class="['badge', statusBadge(req.status_code)]">{{ req.status_code }}</span>
            <span style="font-size: 12px; color: var(--text-muted); min-width: 60px; text-align: right;">
              {{ req.latency_ms }}ms
            </span>
            <span style="font-size: 12px; color: var(--text-muted); min-width: 120px; text-align: right;">
              {{ formatTime(req.timestamp) }}
            </span>
          </div>

          <!-- expanded detail -->
          <div v-if="selectedRequest?.id === req.id" style="margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--border);">
            <div class="detail-grid">
              <div class="detail-item">
                <span class="detail-label">Client IP</span>
                <span class="detail-value">{{ req.client_ip || 'unknown' }}</span>
              </div>
              <div class="detail-item">
                <span class="detail-label">Timestamp</span>
                <span class="detail-value">{{ new Date(req.timestamp).toLocaleString() }}</span>
              </div>
            </div>
            <div v-if="req.request_headers" style="margin-top: 12px;">
              <span class="detail-label">Request Headers</span>
              <pre style="margin-top: 6px; padding: 12px; background: var(--bg-primary); border-radius: var(--radius); font-size: 12px; overflow-x: auto; color: var(--text-secondary);">{{ JSON.stringify(req.request_headers, null, 2) }}</pre>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useInspectorStore } from '@/stores/inspector'
import Sidebar from '@/components/Sidebar.vue'
import type { TunnelRequest } from '@/types'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const inspectorStore = useInspectorStore()
const selectedRequest = ref<TunnelRequest | null>(null)

const tunnelId = computed(() => route.params.tunnelId as string)

onMounted(() => {
  inspectorStore.fetchRequests(tunnelId.value)
})

function refresh() {
  inspectorStore.fetchRequests(tunnelId.value)
}

function methodBadge(method: string): string {
  const map: Record<string, string> = { GET: 'badge-green', POST: 'badge-blue', DELETE: 'badge-red', PUT: 'badge-blue', PATCH: 'badge-blue' }
  return map[method] || 'badge-blue'
}

function statusBadge(code: number): string {
  if (code < 300) return 'badge-green'
  if (code < 400) return 'badge-blue'
  return 'badge-red'
}

function formatTime(ts: string): string {
  return new Date(ts).toLocaleTimeString()
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.request-card {
  cursor: pointer;
  padding: 12px 16px;
}
.request-card:hover {
  border-color: var(--accent-cyan);
}
</style>
