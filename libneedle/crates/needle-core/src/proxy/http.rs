// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

// The HTTP reverse proxy will be implemented in Phase 1b.
// It will accept incoming HTTP requests on tunnel subdomains,
// forward them through the internal TCP listener to the SSH channel,
// and stream the response back to the caller.
