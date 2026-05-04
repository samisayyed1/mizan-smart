<div align="center">
  <img src="apps/frontend/public/logo.svg" alt="Mizan" width="80" height="80">

  <h3 align="center">Mizan</h3>

  <p align="center">
    A Beautiful and Boring Desktop Investment Tracker
    <br />
    <br />
    <!-- TODO: replace with mizan domain when registered -->
    <a href="https://mizan.app">Website</a>
    ·
    <!-- TODO: add Mizan community links (Discord/X) when set up -->
    <a href="https://github.com/samisayyed1/mizan-4/releases">Releases</a>
  </p>
</div>

## Attribution

Mizan is a fork of [Wealthfolio](https://github.com/afadil/wealthfolio)
by Teymz Inc., licensed under AGPL-3.0. Massive thanks to the original
authors. See [NOTICE](./NOTICE) and [CHANGES.md](./CHANGES.md).

## Introduction

**Mizan** is a Beautiful and Boring Investment Tracker, with Local
Data Storage. No Subscriptions, No Cloud.

### ✨ Key Features

- **📊 Portfolio Tracking** - Track your investments across multiple accounts
  and asset types
- **📈 Performance Analytics** - Detailed performance metrics and historical
  analysis
- **💰 Activity Management** - Import and manage all your trading activities
- **🎯 Goal Planning** - Set and track financial goals with allocation
  management
- **🔒 Local Data** - All data stored locally with no cloud dependencies
- **🧩 Extensible** - Powerful addon system for custom functionality
- **🌍 Multi-Currency** - Support for multiple currencies with exchange rate
  management
- **📱 Cross-Platform** - Available on Windows, macOS, and Linux

### 🧩 Addon System

Mizan features a powerful addon system that allows developers to extend
functionality:

- **🔌 Easy Development** - TypeScript SDK with full type safety and hot reload
- **🔒 Secure** - Comprehensive permission system with user consent
- **⚡ High Performance** - Optimized for speed with minimal overhead
- **🎨 UI Integration** - Add custom pages, navigation items, and components
- **📡 Real-time Events** - Listen to portfolio updates, market sync, and user
  actions
- **🗄️ Full Data Access** - Access to accounts, holdings, activities, and market
  data
- **🔐 Secrets Management** - Secure storage for API keys and sensitive data

**Get started building addons:** See the
[Addon Documentation Hub](docs/addons/index.md)

## Roadmap

See [ROADMAP.md](./ROADMAP.md).

## Getting Started

### Prerequisites

Ensure you have the following installed on your machine:

- [Node.js](https://nodejs.org/)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/)
- [Tauri](https://tauri.app/)

### Building from Source

```bash
git clone https://github.com/samisayyed1/mizan-4.git
cd mizan-4
pnpm install
pnpm tauri dev      # development
pnpm tauri build    # production
```

## Folder Structure

```
mizan/
├── apps/                        # Application packages
│   ├── frontend/                # React frontend application
│   ├── tauri/                   # Tauri desktop app (Rust IPC commands)
│   └── server/                  # Axum HTTP server for web mode
├── crates/                      # Rust crates (shared backend logic)
│   ├── core/                    # Core business logic, services, models
│   ├── storage-sqlite/          # SQLite storage layer (Diesel ORM)
│   ├── market-data/             # Market data providers
│   ├── connect/                 # External service integrations
│   └── device-sync/             # Device sync functionality
├── addons/                      # Example addons
├── packages/                    # Shared TypeScript packages
└── docs/                        # Documentation
```

## Contributing

Contributions are welcome! Open an issue or pull request.

## License

This project is licensed under the AGPL-3.0 license. See the [LICENSE](./LICENSE)
file for details.

Mizan is a fork of Wealthfolio. "Wealthfolio" and the Wealthfolio logo are
trademarks of Teymz Inc. and are not used by this fork. See [NOTICE](./NOTICE)
and [CHANGES.md](./CHANGES.md) for fork details.
