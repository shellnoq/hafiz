---
title: Development
description: Contributing to Hafiz
---

# Development

<div class="grid cards" markdown>

-   :material-account-group:{ .lg .middle } __Contributing__

    ---

    How to contribute to Hafiz.

    [:octicons-arrow-right-24: Contributing](contributing.md)

-   :material-road-variant:{ .lg .middle } __Roadmap__

    ---

    Future plans and features.

    [:octicons-arrow-right-24: Roadmap](roadmap.md)

-   :material-history:{ .lg .middle } __Changelog__

    ---

    Version history.

    [:octicons-arrow-right-24: Changelog](changelog.md)

</div>

## Quick Setup

```bash
# Clone
git clone https://github.com/shellnoq/hafiz.git
cd hafiz

# Build
cargo build

# Test
cargo test

# Run
cargo run --bin hafiz-server
```

## Requirements

- Rust 1.75+
- PostgreSQL 13+ (optional)
- Docker (optional)
