<!-- Author : Eshan Roy <eshanized@proton.me> -->
<!-- SPDX-License-Identifier: MIT -->

<template>
  <div class="auth-page">
    <div class="auth-card">
      <h1 class="auth-title">Create your account</h1>
      <p class="auth-subtitle">Start tunneling in seconds</p>

      <div v-if="authStore.error" class="auth-error">{{ authStore.error }}</div>

      <form @submit.prevent="handleRegister">
        <div class="input-group">
          <label for="username">Username</label>
          <input id="username" v-model="username" type="text" class="input-field" placeholder="cool-dev" required />
        </div>

        <div class="input-group">
          <label for="email">Email</label>
          <input id="email" v-model="email" type="email" class="input-field" placeholder="you@example.com" required />
        </div>

        <div class="input-group">
          <label for="password">Password</label>
          <input id="password" v-model="password" type="password" class="input-field" placeholder="strong password" required minlength="8" />
        </div>

        <button type="submit" class="btn btn-primary" :disabled="authStore.loading" style="width: 100%;">
          <span v-if="authStore.loading" class="spinner"></span>
          <span v-else>Create account</span>
        </button>
      </form>

      <div class="auth-footer">
        Already have an account? <router-link to="/login">Sign in</router-link>
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
const username = ref('')
const email = ref('')
const password = ref('')

async function handleRegister() {
  try {
    await authStore.register(email.value, username.value, password.value)
    router.push('/')
  } catch {
    // error handled in store
  }
}
</script>
