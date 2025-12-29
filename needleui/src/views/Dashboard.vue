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
        <h1 class="page-title">Dashboard</h1>
      </div>

      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-label">Active Tunnels</div>
          <div class="stat-value">{{ tunnelsStore.activeCount }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Total Tunnels</div>
          <div class="stat-value">{{ tunnelsStore.tunnels.length }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Account Tier</div>
          <div class="stat-value" style="font-size: 20px; text-transform: capitalize;">
            {{ authStore.user?.tier || 'free' }}
          </div>
        </div>
      </div>

      <div class="page-header">
        <h2 style="font-size: 18px;">Recent Tunnels</h2>
        <router-link to="/tunnels" class="btn btn-ghost">View all</router-link>
      </div>

      <div v-if="tunnelsStore.loading" class="loading">
        <div class="spinner"></div>
      </div>

      <div v-else-if="tunnelsStore.tunnels.length === 0" class="empty-state">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 14a1 1 0 01-.78-1.63l9.9-10.2a.5.5 0 01.86.46l-1.92 6.02A1 1 0 0013 10h7a1 1 0 01.78 1.63l-9.9 10.2a.5.5 0 01-.86-.46l1.92-6.02A1 1 0 0011 14H4z"/>
        </svg>
        <p>No tunnels yet. Create one to get started.</p>
        <router-link to="/tunnels" class="btn btn-primary">Create Tunnel</router-link>
      </div>

      <div v-else class="tunnel-list">
        <div v-for="tunnel in tunnelsStore.tunnels.slice(0, 5)" :key="tunnel.id" class="tunnel-card">
          <div class="tunnel-info">
            <div class="tunnel-subdomain">{{ tunnel.subdomain }}</div>
            <div class="tunnel-url">{{ tunnel.url || `https://${tunnel.subdomain}.needle.dev` }}</div>
          </div>
          <div class="tunnel-status">
            <span :class="['status-dot', tunnel.is_active ? 'active' : 'inactive']"></span>
            <span :class="['badge', tunnel.is_active ? 'badge-green' : 'badge-red']">
              {{ tunnel.is_active ? 'Active' : 'Inactive' }}
            </span>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useTunnelsStore } from '@/stores/tunnels'

const router = useRouter()
const authStore = useAuthStore()
const tunnelsStore = useTunnelsStore()

onMounted(() => {
  tunnelsStore.fetchTunnels()
})

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>
