<!-- Author : Eshan Roy <eshanized@proton.me> -->
<!-- SPDX-License-Identifier: MIT -->

<template>
  <div class="auth-page">
    <div class="auth-card">
      <h1 class="auth-title">Sign in to Needle</h1>
      <p class="auth-subtitle">Expose your local apps to the world</p>

      <div v-if="authStore.error" class="auth-error">{{ authStore.error }}</div>

      <form @submit.prevent="handleLogin">
        <div class="input-group">
          <label for="email">Email</label>
          <input id="email" v-model="email" type="email" class="input-field" placeholder="you@example.com" required />
        </div>

        <div class="input-group">
          <label for="password">Password</label>
          <input id="password" v-model="password" type="password" class="input-field" placeholder="your password" required />
        </div>

        <button type="submit" class="btn btn-primary" :disabled="authStore.loading" style="width: 100%;">
          <span v-if="authStore.loading" class="spinner"></span>
          <span v-else>Sign in</span>
        </button>
      </form>

      <div class="auth-footer">
        Don't have an account? <router-link to="/register">Create one</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const email = ref('')
const password = ref('')

async function handleLogin() {
  try {
    await authStore.login(email.value, password.value)
    router.push('/')
  } catch {
    // error is handled in the store
  }
}
</script>
