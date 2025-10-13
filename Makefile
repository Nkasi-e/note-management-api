# SQLX Makefile — handy commands with docs
# Usage: `make <target>` (run `make help` to list all)

SHELL := /bin/sh

# Helper to load .env into the subshell for each recipe
define LOAD_ENV
    if [ -f .env ]; then set -a; . ./.env; set +a; fi;
endef

.PHONY: help sqlx-install db-create db-ext migrate-new migrate-run migrate-revert migrate-info prepare prepare-online check run env-print watch-install watch-run watch-check watch-test watch-prepare bench-install bench-seed bench-all bench-load bench-stress bench-websocket bench-quick bench-results bench-latest bench-clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | sed 's/:.*## /	- /'

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

# --- Benchmarking Commands ---

bench-install: ## Install benchmarking tools (wrk and k6)
	@echo "📦 Installing benchmarking tools..."
	@if command -v brew >/dev/null 2>&1; then \
		echo "  Installing wrk..."; \
		brew install wrk 2>/dev/null || echo "  ✓ wrk already installed"; \
		echo "  Installing k6..."; \
		brew install k6 2>/dev/null || echo "  ✓ k6 already installed"; \
		echo "✅ Tools installed!"; \
	else \
		echo "❌ Homebrew not found. Install manually:"; \
		echo "   wrk: https://github.com/wg/wrk"; \
		echo "   k6:  https://k6.io/docs/getting-started/installation"; \
	fi

bench-seed: ## Seed test data for benchmarking (10 users, 20 tasks each)
	@echo "🌱 Seeding test data..."
	@cd benchmarks && ./scripts/seed-data.sh

bench-all: ## Run complete benchmark suite (wrk + k6)
	@echo "🚀 Running complete benchmark suite..."
	@cd benchmarks && ./scripts/run-benchmarks.sh

bench-load: ## Run k6 load test (comprehensive user scenarios)
	@echo "📊 Running k6 load test..."
	@k6 run -e BASE_URL=http://localhost:3000 benchmarks/k6/load-test.js

bench-stress: ## Run k6 stress test (find breaking point, ~30 min)
	@echo "💪 Running k6 stress test (this takes ~30 minutes)..."
	@k6 run -e BASE_URL=http://localhost:3000 benchmarks/k6/stress-test.js

bench-websocket: ## Run k6 WebSocket test (arena allocation optimization)
	@echo "🔌 Running k6 WebSocket test..."
	@k6 run -e WS_URL=ws://localhost:3000 benchmarks/k6/websocket-test.js

bench-quick: ## Quick benchmark (10s duration, 50 connections)
	@echo "⚡️ Running quick benchmark..."
	@wrk -t4 -c50 -d10s http://localhost:3000/health

bench-results: ## List all benchmark results
	@echo "📊 Benchmark results:"
	@ls -lht benchmarks/results/ 2>/dev/null | head -10 || echo "No results yet. Run 'make bench-all' first."

bench-latest: ## Show latest benchmark results summary
	@echo "📊 Latest benchmark results:"
	@if [ -d benchmarks/results ]; then \
		LATEST=$$(ls -t benchmarks/results/ | head -1 2>/dev/null); \
		if [ -n "$$LATEST" ]; then \
			echo ""; \
			echo "Directory: benchmarks/results/$$LATEST"; \
			echo ""; \
			for file in benchmarks/results/$$LATEST/*.txt; do \
				if [ -f "$$file" ]; then \
					echo "═══════════════════════════════════════"; \
					echo "📄 $$(basename $$file)"; \
					echo "═══════════════════════════════════════"; \
					grep -E "Requests/sec:|http_req_duration|Latency" "$$file" | head -5; \
					echo ""; \
				fi; \
			done; \
		else \
			echo "No results found. Run 'make bench-all' first."; \
		fi; \
	else \
		echo "No results directory. Run 'make bench-all' first."; \
	fi

bench-clean: ## Remove all benchmark results
	@echo "🧹 Cleaning benchmark results..."
	@rm -rf benchmarks/results/
	@echo "✅ Benchmark results cleaned!"


