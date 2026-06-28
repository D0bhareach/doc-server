# doc-server

A lightweight, high-performance local documentation server and dashboard built with Rust and Axum. 

This utility dynamically scans your pre-compiled `target/doc` directory, automatically builds an interactive nested tree dashboard of your crates, and patches relative asset routing—eliminating the broken styles and 404 errors common when serving Cargo docs via standard static file servers.

## Features

- **Dynamic Discovery:** Automatically indexes and lists newly compiled workspace crates on refresh without manual route registrations.
- **Accordion Navigation:** Built-in semantic HTML tree layout with smooth, hardware-accelerated CSS disclosure controls.
- **Smart Asset Routing:** Custom Axum fallback layers intercepting pathing conflicts to seamlessly serve underlying `static.files` dependency assets.
- **Supply Chain Security:** Strictly bound by corporate compliance policies utilizing `cargo-deny` configurations.

## Architecture Details

Standard `cargo doc` outputs tightly coupled relative resource structures. This project isolates routing logic into two distinct tiers:
1. **The Core Asset Service:** Uses `tower_http::services::ServeDir` to stream underlying structural HTML, JS, and CSS files on-demand.
2. **The Interceptor Fallback:** A unified Axum handler that catches root-level hits, bypassing directory index limitations to dynamically generate a clean landing dashboard.

## Quick Start

### Prerequisites
Ensure you have the Rust toolchain installed and a compiled target documentation folder:

```bash
cargo doc
```

### Installation & Running

1. Clone the repository:

```bash
git clone git@github.com:D0bhareach/doc-server.git
cd doc-server

```

2. Run the development server:
```bash
cargo run --release

```

3. Open your browser and navigate to: `http://localhost:8080/doc/`

## Security Policy

This repository enforces strict dependency and licensing validations. Security pipelines are actively evaluated via **cargo-deny**: `cargo deny check`. 

