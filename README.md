# grav-sim

Browser-based n-body gravity simulator. The physics is written in Rust and compiled to WebAssembly, and the frontend is Svelte + Vite.

## Requirements

- [Rust](https://rustup.rs) (via rustup)
- [Node.js](https://nodejs.org) with npm

Add the WebAssembly target and install wasm-pack:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Build & run

All commands are run from the `web/` directory:

```sh
cd web
npm install
```

| Command | What it does |
|---|---|
| `npm start` | Compiles the Rust crate to WASM, then starts the dev server |
| `npm run build` | Compiles the Rust crate to WASM, then builds the site into `web/dist/` |
| `npm run wasm` | Compiles the Rust crate to WASM only (output in `web/pkg/`) |
| `npm run dev` | Starts the dev server without recompiling Rust |

The dev server doesn't watch Rust files. After changing Rust code, run `npm run wasm`, and the page will reload.
