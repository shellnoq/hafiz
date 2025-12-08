# Hafiz Admin UI

A modern, responsive web interface for managing Hafiz - built with Leptos (Rust).

## Features

- 📊 **Dashboard** - Overview statistics, recent buckets, quick actions
- 🪣 **Bucket Management** - Create, browse, delete buckets
- 📁 **Object Browser** - Navigate folders, view files with icons, encryption status
- 👥 **User Management** - Create users, manage credentials
- ⚙️ **Settings** - Server config, encryption, lifecycle settings
- 🔐 **Authentication** - Login with access/secret keys
- 🌙 **Dark Theme** - Modern dark UI with Tailwind CSS

## Architecture

```
hafiz-admin/
├── src/
│   ├── lib.rs            # WASM entry point
│   ├── app.rs            # Router & layouts
│   ├── api/              # API client
│   │   ├── client.rs     # HTTP requests
│   │   └── types.rs      # Response types
│   ├── components/       # Reusable UI
│   │   ├── sidebar.rs
│   │   ├── header.rs
│   │   ├── table.rs
│   │   ├── modal.rs
│   │   ├── stats.rs
│   │   └── button.rs
│   └── pages/            # Page components
│       ├── dashboard.rs
│       ├── buckets.rs
│       ├── objects.rs
│       ├── users.rs
│       ├── settings.rs
│       └── not_found.rs
├── index.html            # HTML template
├── Trunk.toml            # Build config
└── Cargo.toml
```

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk (build tool)
cargo install trunk
```

## Development

```bash
cd crates/hafiz-admin

# Start development server with hot reload
trunk serve

# Open http://localhost:8080
```

## Production Build

```bash
# Build optimized WASM
trunk build --release

# Output in dist/ directory
ls dist/
# index.html
# hafiz_admin-*.wasm
# hafiz_admin-*.js
```

## Deployment Options

### 1. Standalone (Recommended for Air-Gapped)

The `dist/` folder contains everything needed. Serve with any HTTP server:

```bash
# Using Python
cd dist && python3 -m http.server 8080

# Using nginx
cp -r dist/* /var/www/hafiz-admin/
```

### 2. Embedded in Hafiz Binary

The Admin API can serve the UI files directly:

```rust
// In hafiz-s3-api
#[cfg(feature = "admin-ui")]
fn embed_admin_ui() -> Router {
    Router::new()
        .route("/", get(|| async { Html(include_str!("../admin/dist/index.html")) }))
        // ... serve WASM and JS
}
```

### 3. Docker

```dockerfile
FROM nginx:alpine
COPY dist/ /usr/share/nginx/html/
EXPOSE 80
```

## Configuration

### API Endpoint

By default, the UI proxies `/api/*` requests to the backend. Configure in production:

```javascript
// In localStorage or environment
window.HAFIZ_API_URL = "https://storage.example.com:9001";
```

### Authentication

Credentials are stored in localStorage:
- `hafiz_access_key` - S3 Access Key
- `hafiz_secret_key` - S3 Secret Key

## Screenshots

```
┌────────────────────────────────────────────────────────────┐
│  🪣 Hafiz          [Search...]              👤 Admin  │
├──────────┬─────────────────────────────────────────────────┤
│          │                                                 │
│ Dashboard│  Dashboard                                      │
│ Buckets  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
│ Users    │  │ 5      │ │ 1,247  │ │ 2.5 GB │ │ 3      │   │
│ ──────── │  │Buckets │ │Objects │ │Storage │ │Users   │   │
│ Settings │  └────────┘ └────────┘ └────────┘ └────────┘   │
│          │                                                 │
│          │  Recent Buckets          Quick Actions          │
│          │  ┌─────────────────┐    ┌─────────────────┐    │
│          │  │ 🪣 documents    │    │ + Create Bucket │    │
│          │  │ 🪣 backups      │    │ 👤 Add User     │    │
│          │  │ 🪣 media        │    │ ⚙ Settings      │    │
│          │  └─────────────────┘    └─────────────────┘    │
└──────────┴─────────────────────────────────────────────────┘
```

## Tech Stack

- **Leptos 0.6** - Reactive Rust framework
- **Tailwind CSS** - Utility-first styling
- **gloo-net** - HTTP client for WASM
- **leptos_router** - Client-side routing

## Why Leptos?

1. **Single Binary** - Compiles to WASM, no Node.js needed
2. **Air-Gapped Ready** - No npm, no external dependencies at runtime
3. **Type Safety** - Full Rust type checking
4. **Performance** - Near-native speed, small bundle size
5. **Security** - Minimal supply chain attack surface

## API Integration

The UI communicates with Hafiz Admin API:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/stats` | GET | Dashboard statistics |
| `/api/v1/buckets` | GET/POST | List/create buckets |
| `/api/v1/buckets/{name}` | GET/DELETE | Bucket details |
| `/api/v1/buckets/{name}/objects` | GET | List objects |
| `/api/v1/users` | GET/POST | List/create users |
| `/api/v1/server/info` | GET | Server information |

## Contributing

1. Fork the repository
2. Create feature branch
3. Run `trunk serve` for development
4. Submit pull request

## License

Apache 2.0
