# ALICE-VR-SaaS

VR Runtime API — session management, 6DoF tracking, rendering pipeline control, and comfort analytics via the ALICE SaaS architecture.

## Architecture

```
Client
  └─ API Gateway (:8146) — JWT auth, rate limiting, proxy
       └─ Core Engine (:9146) — session manager, tracker, renderer, comfort
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| POST | /api/v1/vr/session | Create or manage VR session |
| POST | /api/v1/vr/tracking | Submit 6DoF tracking data |
| POST | /api/v1/vr/render | Render frame request |
| GET | /api/v1/vr/comfort | Comfort and motion-sickness analytics |
| GET | /api/v1/vr/stats | Request statistics |

## Quick Start

```bash
cd services/core-engine && cargo run
cd services/api-gateway && cargo run
```

## License

AGPL-3.0-or-later
