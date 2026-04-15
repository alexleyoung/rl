db := "sqlite:./data/rl.db"

# Run dev server
run:
    DATABASE_URL={{db}} cargo run

# Build debug binary
build:
    DATABASE_URL={{db}} cargo build

# Build release binary
release:
    DATABASE_URL={{db}} cargo build --release

# Rebuild the CodeMirror + vim JS bundle
bundle:
    ./node_modules/.bin/esbuild cm-build/build.js --bundle --format=iife \
        --outfile=static/codemirror.bundle.js --minify

# Run migrations against the local DB
migrate:
    DATABASE_URL={{db}} cargo sqlx migrate run

# Regenerate sqlx offline query cache (run after changing SQL queries)
prepare:
    DATABASE_URL={{db}} cargo sqlx prepare

# Full rebuild: bundle JS then build Rust
all: bundle build
