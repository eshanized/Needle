<!-- Author : Eshan Roy <eshanized@proton.me> -->
<!-- SPDX-License-Identifier: MIT -->

<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-brand">
        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z"/>
          <path d="M2 17l10 5 10-5"/>
          <path d="M2 12l10 5 10-5"/>
        </svg>
        Needle
      </div>
      <nav class="sidebar-nav">
        <router-link to="/" class="sidebar-link" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
          </svg>
          Dashboard
        </router-link>
        <router-link to="/tunnels" class="sidebar-link" active-class="active">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 14a1 1 0 01-.78-1.63l9.9-10.2a.5.5 0 01.86.46l-1.92 6.02A1 1 0 0013 10h7a1 1 0 01.78 1.63l-9.9 10.2a.5.5 0 01-.86-.46l1.92-6.02A1 1 0 0011 14H4z"/>
          </svg>
          Tunnels
        </router-link>
      </nav>
      <div class="sidebar-footer">
        <button class="btn btn-ghost" style="width: 100%;" @click="handleLogout">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
            <polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/>
          </svg>
          Sign out
        </button>
      </div>
    </aside>

    <main class="main-content">
      <div class="page-header">
        <div>
          <router-link to="/tunnels" style="font-size: 13px; color: var(--text-muted);">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align: -2px;">
              <polyline points="15 18 9 12 15 6"/>
            </svg>
            Back to tunnels
          </router-link>
          <h1 class="page-title" style="margin-top: 8px;">{{ subdomain }}</h1>
        </div>
        <span :class="['badge', tunnel?.is_active ? 'badge-green' : 'badge-red']">
          {{ tunnel?.is_active ? 'Active' : 'Inactive' }}
        </span>
      </div>

      <div v-if="!tunnel" class="loading">
        <div class="spinner"></div>
      </div>

      <template v-else>
        <div class="card" style="margin-bottom: 24px;">
          <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em;">
            Tunnel Details
          </h3>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">Subdomain</span>
              <span class="detail-value">{{ tunnel.subdomain }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">URL</span>
              <span class="detail-value">{{ tunnel.url || `https://${tunnel.subdomain}.needle.dev` }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Protocol</span>
              <span class="detail-value">{{ tunnel.protocol.toUpperCase() }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Target Port</span>
              <span class="detail-value">{{ tunnel.target_port }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Created</span>
              <span class="detail-value">{{ formatDate(tunnel.created_at) }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Last Active</span>
              <span class="detail-value">{{ formatDate(tunnel.last_active) }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Persistent</span>
              <span class="detail-value">{{ tunnel.is_persistent ? 'Yes' : 'No' }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Custom Domain</span>
              <span class="detail-value">{{ tunnel.custom_domain || 'None' }}</span>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em;">
            Connection
          </h3>
          <div class="detail-item" style="margin-bottom: 16px;">
            <span class="detail-label">SSH Command</span>
            <code class="detail-value" style="background: var(--bg-primary); padding: 12px 16px; border-radius: var(--radius); display: block; margin-top: 6px;">
              ssh -R 80:localhost:{{ tunnel.target_port }} needle.dev
            </code>
          </div>
        </div>
      </template>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useTunnelsStore } from '@/stores/tunnels'
import type { Tunnel } from '@/types'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const tunnelsStore = useTunnelsStore()

const subdomain = computed(() => route.params.subdomain as string)
const tunnel = computed<Tunnel | undefined>(() =>
  tunnelsStore.tunnels.find((t) => t.subdomain === subdomain.value)
)

onMounted(async () => {
  if (tunnelsStore.tunnels.length === 0) {
    await tunnelsStore.fetchTunnels()
  }
})

function formatDate(dateStr: string): string {
  if (!dateStr) return 'N/A'
  return new Date(dateStr).toLocaleString()
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>
