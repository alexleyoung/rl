db := "sqlite:./data/rl.db"
export TS_RS_EXPORT_DIR := justfile_directory()

# Run dev server
run:
    DATABASE_URL={{db}} cargo run -p rl-server

# Build debug binary
build:
    DATABASE_URL={{db}} cargo build -p rl-server

# Build release binary
release:
    DATABASE_URL={{db}} cargo build --release -p rl-server

# Rebuild the CodeMirror + vim JS bundle (legacy; removed in phase 6)
bundle:
    ./node_modules/.bin/esbuild cm-build/build.js --bundle --format=iife \
        --outfile=static/codemirror.bundle.js --minify

# Run Svelte client dev server
client:
    cd client && bun run dev

# Run server + client in parallel (prefer two terminals: just server + just client)
dev:
    DATABASE_URL={{db}} cargo run -p rl-server &
    (cd client && bun run dev); kill %1 2>/dev/null || true

# Run migrations against the local DB
migrate:
    DATABASE_URL={{db}} cargo sqlx migrate run

# Regenerate sqlx offline query cache (run after changing SQL queries)
prepare:
    DATABASE_URL={{db}} cargo sqlx prepare --workspace

# Regenerate TypeScript bindings from Rust DTOs (ts-rs)
types:
    DATABASE_URL={{db}} cargo test -p rl-server export_bindings_

# Full rebuild: types + server + client
all: types build
    cd client && bun install --frozen-lockfile && bun run build
