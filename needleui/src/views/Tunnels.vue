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
        <h1 class="page-title">Tunnels</h1>
        <button class="btn btn-primary" @click="showCreateModal = true">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          New Tunnel
        </button>
      </div>

      <!-- Create modal -->
      <div v-if="showCreateModal" class="card" style="margin-bottom: 24px;">
        <h3 style="margin-bottom: 16px; font-size: 16px;">Create a new tunnel</h3>
        <form @submit.prevent="handleCreate">
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
            <div class="input-group">
              <label for="port">Local port</label>
              <input id="port" v-model.number="newTunnel.target_port" type="number" class="input-field" placeholder="3000" required min="1" max="65535" />
            </div>
            <div class="input-group">
              <label for="protocol">Protocol</label>
              <select id="protocol" v-model="newTunnel.protocol" class="input-field">
                <option value="http">HTTP</option>
                <option value="tcp">TCP</option>
              </select>
            </div>
          </div>
          <div style="display: flex; gap: 8px; justify-content: flex-end;">
            <button type="button" class="btn btn-ghost" @click="showCreateModal = false">Cancel</button>
            <button type="submit" class="btn btn-primary">Create</button>
          </div>
        </form>
      </div>

      <div v-if="tunnelsStore.loading" class="loading">
        <div class="spinner"></div>
      </div>

      <div v-else-if="tunnelsStore.tunnels.length === 0" class="empty-state">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 14a1 1 0 01-.78-1.63l9.9-10.2a.5.5 0 01.86.46l-1.92 6.02A1 1 0 0013 10h7a1 1 0 01.78 1.63l-9.9 10.2a.5.5 0 01-.86-.46l1.92-6.02A1 1 0 0011 14H4z"/>
        </svg>
        <p>No tunnels yet. Click "New Tunnel" to get started.</p>
      </div>

      <div v-else class="tunnel-list">
        <div v-for="tunnel in tunnelsStore.tunnels" :key="tunnel.id" class="tunnel-card">
          <router-link :to="`/tunnels/${tunnel.subdomain}`" style="flex: 1; text-decoration: none;">
            <div class="tunnel-info">
              <div class="tunnel-subdomain">{{ tunnel.subdomain }}</div>
              <div class="tunnel-url">{{ tunnel.url || `https://${tunnel.subdomain}.needle.dev` }}</div>
            </div>
          </router-link>
          <div class="tunnel-status">
            <span :class="['status-dot', tunnel.is_active ? 'active' : 'inactive']"></span>
            <span :class="['badge', tunnel.is_active ? 'badge-green' : 'badge-red']">
              {{ tunnel.is_active ? 'Active' : 'Inactive' }}
            </span>
          </div>
          <div class="tunnel-actions">
            <button class="btn btn-danger" style="padding: 6px 12px; font-size: 13px;" @click="handleDelete(tunnel.subdomain)">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
              </svg>
              Delete
            </button>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useTunnelsStore } from '@/stores/tunnels'

const router = useRouter()
const authStore = useAuthStore()
const tunnelsStore = useTunnelsStore()
const showCreateModal = ref(false)
const newTunnel = reactive({ target_port: 3000, protocol: 'http' })

onMounted(() => {
  tunnelsStore.fetchTunnels()
})

async function handleCreate() {
  await tunnelsStore.createTunnel(newTunnel)
  showCreateModal.value = false
}

async function handleDelete(subdomain: string) {
  await tunnelsStore.deleteTunnel(subdomain)
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>
