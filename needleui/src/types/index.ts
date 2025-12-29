// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

export interface User {
  id: string
  email: string
  username: string
  tier: string
  created_at: string
}

export interface Tunnel {
  id: string
  user_id: string
  subdomain: string
  custom_domain: string | null
  target_port: number
  protocol: string
  is_active: boolean
  is_persistent: boolean
  created_at: string
  last_active: string
  url?: string
}

export interface TunnelRequest {
  id: string
  tunnel_id: string
  method: string
  path: string
  status_code: number
  latency_ms: number
  request_headers: Record<string, string> | null
  response_headers: Record<string, string> | null
  timestamp: string
}

export interface AuthResponse {
  token: string
  user: User
}

export interface CreateTunnelRequest {
  subdomain?: string
  target_port: number
  protocol?: string
  is_persistent?: boolean
}

export interface ApiError {
  error: string
}
