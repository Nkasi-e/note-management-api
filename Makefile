# SQLX Makefile — handy commands with docs
# Usage: `make <target>` (run `make help` to list all)

SHELL := /bin/sh

# Helper to load .env into the subshell for each recipe
define LOAD_ENV
    if [ -f .env ]; then set -a; . ./.env; set +a; fi;
endef

.PHONY: help sqlx-install db-create db-ext migrate-new migrate-run migrate-revert migrate-info prepare prepare-online check run env-print watch-install watch-run watch-check watch-test watch-prepare

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | sed -E 's/:.*?## /\t- /'

sqlx-install: ## Install or update sqlx CLI (Postgres + rustls)
	cargo install sqlx-cli --no-default-features --features rustls,postgres --force

db-create: ## Create database from DATABASE_URL (falls back to `noteapi`)
	@$(LOAD_ENV) \
	DB_NAME=$${DATABASE_URL##*/}; \
	DB_NAME=$${DB_NAME:-noteapi}; \
	createdb $$DB_NAME 2>/dev/null || true; \
	echo "createdb $$DB_NAME (if missing)"

db-ext: ## Enable pgcrypto extension (UUID generation)
	@$(LOAD_ENV) \
	DB_NAME=$${DATABASE_URL##*/}; \
	psql -d $$DB_NAME -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"

migrate-new: ## Create a new migration: make migrate-new name=create_tasks
	@test -n "$(name)" || (echo "Usage: make migrate-new name=<migration_name>" && exit 1)
	sqlx migrate add -r $(name)

migrate-run: ## Apply all pending migrations
	@$(LOAD_ENV) \
	sqlx migrate run

migrate-revert: ## Revert last applied migration (interactive)
	@$(LOAD_ENV) \
	sqlx migrate revert

migrate-info: ## Show migration status
	@$(LOAD_ENV) \
	sqlx migrate info

prepare: ## Regenerate SQLX offline metadata (.sqlx) using SQLX_OFFLINE=true
	@$(LOAD_ENV) \
	export SQLX_OFFLINE=true; \
	cargo sqlx prepare -- --bin note-task-api

prepare-online: ## Regenerate SQLX metadata using live DB (SQLX_OFFLINE unset)
	@$(LOAD_ENV) \
	cargo sqlx prepare -- --bin note-task-api

check: ## Build (type-check) the project
	cargo check

run: ## Run the API server (dotenv is loaded in main.rs)
	cargo run

env-print: ## Print key env vars seen by the shell
	@$(LOAD_ENV) \
	echo APP_HOST=$${APP_HOST:-unset}; \
	echo APP_PORT=$${APP_PORT:-unset}; \
	echo DATABASE_URL=$${DATABASE_URL:-unset}; \
	echo RUST_LOG=$${RUST_LOG:-unset}

# --- Nodemon-like dev workflow using cargo-watch ---
watch-install: ## Install cargo-watch (file-watching like nodemon)
	cargo install cargo-watch --force

watch-run: ## Rebuild & restart on changes (src/, migrations/)
	cargo watch -q -c -w src -w migrations -x 'run'

watch-check: ## Fast feedback: type-check on every change
	cargo watch -q -c -w src -x 'check'

watch-test: ## Re-run tests on changes
	cargo watch -q -c -w src -x 'test'

watch-prepare: ## Re-generate SQLX metadata when queries/migrations change
	cargo watch -q -c -w src -w migrations -s 'cargo sqlx prepare -- --bin note-task-api'


