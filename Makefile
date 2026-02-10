# Needle Project Makefile
# Author: Eshan Roy <eshanized@proton.me>
# SPDX-License-Identifier: MIT

.PHONY: help all clean build build-backend build-frontend dev dev-backend dev-frontend test test-backend test-frontend docker-build docker-up docker-down docker-clean install install-backend install-frontend format format-backend format-frontend lint lint-backend lint-frontend check

# Default target
help:
	@echo "Needle Project - Available Make Targets"
	@echo "========================================"
	@echo ""
	@echo "Build Commands:"
	@echo "  make all              - Build everything (backend + frontend)"
	@echo "  make build            - Alias for 'make all'"
	@echo "  make build-backend    - Build Rust backend (release mode)"
	@echo "  make build-frontend   - Build Vue.js frontend"
	@echo ""
	@echo "Development Commands:"
	@echo "  make dev              - Run both backend and frontend in dev mode"
	@echo "  make dev-backend      - Run Rust backend server in dev mode"
	@echo "  make dev-frontend     - Run Vue.js frontend dev server"
	@echo ""
	@echo "Testing Commands:"
	@echo "  make test             - Run all tests (backend + frontend)"
	@echo "  make test-backend     - Run Rust backend tests"
	@echo "  make test-frontend    - Run frontend tests"
	@echo ""
	@echo "Code Quality Commands:"
	@echo "  make check            - Run all checks (format + lint + test)"
	@echo "  make format           - Format all code (backend + frontend)"
	@echo "  make format-backend   - Format Rust code with cargo fmt"
	@echo "  make format-frontend  - Format frontend code (if configured)"
	@echo "  make lint             - Lint all code (backend + frontend)"
	@echo "  make lint-backend     - Lint Rust code with cargo clippy"
	@echo "  make lint-frontend    - Lint frontend code (if configured)"
	@echo ""
	@echo "Docker Commands:"
	@echo "  make docker-build     - Build Docker images"
	@echo "  make docker-up        - Start services with docker-compose"
	@echo "  make docker-down      - Stop services"
	@echo "  make docker-clean     - Stop and remove containers, networks, and volumes"
	@echo ""
	@echo "Installation Commands:"
	@echo "  make install          - Install dependencies (backend + frontend)"
	@echo "  make install-backend  - Install Rust backend dependencies"
	@echo "  make install-frontend - Install frontend dependencies"
	@echo ""
	@echo "Cleanup Commands:"
	@echo "  make clean            - Clean all build artifacts"
	@echo ""

# Build targets
all: build-backend build-frontend

build: all

build-backend:
	@echo "Building Rust backend (release mode)..."
	cd libneedle && cargo build --release

build-frontend:
	@echo "Building Vue.js frontend..."
	cd needleui && npm run build

# Development targets
dev-backend:
	@echo "Starting Rust backend server..."
	cd libneedle && cargo run

dev-frontend:
	@echo "Starting Vue.js frontend dev server..."
	cd needleui && npm run dev

dev:
	@echo "Starting both backend and frontend in dev mode..."
	@echo "Note: Run 'make dev-backend' and 'make dev-frontend' in separate terminals"
	@echo "Or use docker-compose for integrated development"

# Test targets
test: test-backend test-frontend

test-backend:
	@echo "Running Rust backend tests..."
	cd libneedle && cargo test

test-frontend:
	@echo "Running frontend tests..."
	@echo "Note: Frontend tests not configured yet"
	# cd needleui && npm test

# Code quality targets
check: format lint test

format: format-backend format-frontend

format-backend:
	@echo "Formatting Rust code..."
	cd libneedle && cargo fmt

format-frontend:
	@echo "Formatting frontend code..."
	@echo "Note: Frontend formatter not configured yet"
	# cd needleui && npm run format

lint: lint-backend lint-frontend

lint-backend:
	@echo "Linting Rust code..."
	cd libneedle && cargo clippy -- -D warnings

lint-frontend:
	@echo "Linting frontend code..."
	@echo "Note: Frontend linter not configured yet"
	# cd needleui && npm run lint

# Docker targets
docker-build:
	@echo "Building Docker images..."
	docker-compose build

docker-up:
	@echo "Starting services with docker-compose..."
	docker-compose up -d

docker-down:
	@echo "Stopping services..."
	docker-compose down

docker-clean:
	@echo "Cleaning up Docker resources..."
	docker-compose down -v --remove-orphans

# Installation targets
install: install-backend install-frontend

install-backend:
	@echo "Installing Rust backend dependencies..."
	cd libneedle && cargo fetch

install-frontend:
	@echo "Installing frontend dependencies..."
	cd needleui && npm install

# Clean targets
clean:
	@echo "Cleaning build artifacts..."
	cd libneedle && cargo clean
	cd needleui && rm -rf dist node_modules/.vite
	@echo "Clean complete!"
