<!-- Author : Eshan Roy <eshanized@proton.me> -->
<!-- SPDX-License-Identifier: MIT -->

<template>
  <div class="app-layout">
    <Sidebar :user-email="authStore.user?.email" @logout="handleLogout" />

    <main class="main-content">
      <div class="page-header">
        <h1 class="page-title">Settings</h1>
      </div>

      <div class="card" style="margin-bottom: 24px;">
        <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em;">
          Account
        </h3>
        <div class="detail-grid">
          <div class="detail-item">
            <span class="detail-label">Email</span>
            <span class="detail-value">{{ authStore.user?.email || 'N/A' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">Username</span>
            <span class="detail-value">{{ authStore.user?.username || 'N/A' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">Tier</span>
            <span class="detail-value" style="text-transform: capitalize;">{{ authStore.user?.tier || 'free' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">Member since</span>
            <span class="detail-value">{{ formatDate(authStore.user?.created_at) }}</span>
          </div>
        </div>
      </div>

      <div class="card" style="margin-bottom: 24px;">
        <h3 style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em;">
          API Keys
        </h3>
        <p style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px;">
          Use API keys to authenticate tunnel connections without entering your password.
        </p>
        <button class="btn btn-primary">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          Generate API Key
        </button>
      </div>

      <div class="card">
        <h3 style="font-size: 14px; color: var(--accent-red); margin-bottom: 16px; text-transform: uppercase; letter-spacing: 0.05em;">
          Danger Zone
        </h3>
        <p style="font-size: 14px; color: var(--text-muted); margin-bottom: 16px;">
          Deleting your account will terminate all active tunnels and remove all data permanently.
        </p>
        <button class="btn btn-danger">Delete Account</button>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import Sidebar from '@/components/Sidebar.vue'

const router = useRouter()
const authStore = useAuthStore()

function formatDate(dateStr?: string): string {
  if (!dateStr) return 'N/A'
  return new Date(dateStr).toLocaleDateString()
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}
</script>
