// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

import axios from 'axios'

function getBaseUrl(): string {
    return import.meta.env.VITE_API_URL || 'http://localhost:3000'
}

const api = axios.create({
    baseURL: getBaseUrl(),
    headers: { 'Content-Type': 'application/json' },
})

api.interceptors.request.use((config) => {
    const token = localStorage.getItem('needle_token')
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }
    return config
})

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            localStorage.removeItem('needle_token')
            window.location.href = '/login'
        }
        return Promise.reject(error)
    }
)

export { api }
