# doc-server

A lightweight, high-performance local documentation server and dashboard built with Rust and Axum. 

This utility dynamically scans your pre-compiled `target/doc` directory, automatically builds an interactive nested tree dashboard of your crates, and patches relative asset routing—eliminating the broken styles and 404 errors common when serving Cargo docs via standard static file servers.

## The Problem & Motivation

When developing software inside an isolated, headless Linux Virtual Machine (VM), accessing standard Rust documentation poses a unique workflow challenge:
1. **Headless Environment:** The guest VM lacks a graphical environment and a web browser, making it impossible to read local `cargo doc` outputs directly inside the machine.
2. **Offline Requirements:** Development often takes place in environments without stable internet connections, making external documentation mirrors unreliable or unavailable.
3. **The Shared-Folder Security Risk:** Binding the VM's target directory directly to the host machine's filesystem introduces significant security vulnerabilities. Shared file-system drivers (such as VirtFS or VirtualBox Shared Folders) are historically prone to **Guest-to-Host Virtualization Breakouts**, **Directory Traversal via malicious Symlinks**, and hypervisor memory corruption exploits.

### The Security Vulnerabilities

By choosing a isolated TCP network bind over a shared directory mount (like VirtFS, VirtualBox Shared Folders, or 9p mounts), you are actively protecting your host machine from several critical virtualization breakout vectors:

1. Directory Traversal & Symlink Attacks: If a guest VM is compromised, an attacker can craft malicious symbolic links inside a shared directory that point to critical host system files (like /etc/shadow or ~/.ssh/id_rsa). When the host OS reads the shared mount, it can be tricked into exposing or overwriting its own files.
2. Shared Memory Heap Exploits: Guest-host file sharing mechanisms often rely on shared driver memory regions (like vhost-user-fs). Over the years, numerous CVEs (vulnerabilities) have emerged where a compromised guest triggers an out-of-bounds read/write in the host's hypervisor process via these shared buffers, resulting in a complete VM Breakout.
3. Cache Invalidation Side-Channels: Shared folders require complex file-system caching synchronization between the host kernel and guest kernel, which can introduce timing side-channel leaks.

### The Solution

`doc_server` bridges this gap safely by containing the entire build ecosystem inside the guest VM. Instead of exposing files, it serves your compiled `target/doc` directory over a secure, isolated **TCP network socket port** bound to the host. 

It automatically builds an interactive nested tree dashboard of your crates and patches relative asset routing—eliminating the broken styles and 404 errors common when serving Cargo docs via standard static file servers.

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

