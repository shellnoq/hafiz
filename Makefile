.PHONY: build run test clean docker docker-run docker-push help

# Variables
VERSION ?= $(shell grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
DOCKER_REGISTRY ?= ghcr.io
DOCKER_IMAGE ?= shellnoq/hafiz
DOCKER_TAG ?= $(VERSION)

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build --release

run: ## Run the server
	cargo run --release --bin hafiz-server

test: ## Run tests
	cargo test --all

clean: ## Clean build artifacts
	cargo clean
	rm -rf data/

lint: ## Run clippy
	cargo clippy --all -- -D warnings

fmt: ## Format code
	cargo fmt --all

check: ## Check code (lint + format)
	cargo fmt --all -- --check
	cargo clippy --all -- -D warnings

docker: ## Build Docker image
	docker build -t $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG) .
	docker tag $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG) $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):latest

docker-run: ## Run Docker container
	docker run -d \
		--name hafiz \
		-p 9000:9000 \
		-p 9001:9001 \
		-v hafiz-data:/data \
		-e HAFIZ_ROOT_ACCESS_KEY=hafizadmin \
		-e HAFIZ_ROOT_SECRET_KEY=hafizadmin \
		$(DOCKER_REGISTRY)/$(DOCKER_IMAGE):latest

docker-stop: ## Stop Docker container
	docker stop hafiz || true
	docker rm hafiz || true

docker-logs: ## Show Docker logs
	docker logs -f hafiz

docker-push: docker ## Push Docker image to registry
	docker push $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG)
	docker push $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):latest

docker-buildx: ## Build multi-arch Docker image
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		-t $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):$(DOCKER_TAG) \
		-t $(DOCKER_REGISTRY)/$(DOCKER_IMAGE):latest \
		--push .

compose-up: ## Start with docker-compose
	docker-compose up -d

compose-down: ## Stop docker-compose
	docker-compose down

compose-logs: ## Show docker-compose logs
	docker-compose logs -f

compose-postgres: ## Start with PostgreSQL
	docker-compose --profile postgres up -d

compose-full: ## Start all services (postgres + monitoring)
	docker-compose --profile full up -d

dev: ## Run in development mode
	HAFIZ_LOG_LEVEL=debug cargo run --bin hafiz-server

dev-watch: ## Run with auto-reload
	cargo watch -x 'run --bin hafiz-server'

install: build ## Install binaries
	cargo install --path crates/hafiz-server
	cargo install --path crates/hafiz-cli

# AWS CLI test commands
test-s3-create-bucket: ## Test: Create bucket
	aws --endpoint-url http://localhost:9000 s3 mb s3://test-bucket

test-s3-list-buckets: ## Test: List buckets
	aws --endpoint-url http://localhost:9000 s3 ls

test-s3-upload: ## Test: Upload file
	echo "Hello Hafiz!" > /tmp/test.txt
	aws --endpoint-url http://localhost:9000 s3 cp /tmp/test.txt s3://test-bucket/

test-s3-list-objects: ## Test: List objects
	aws --endpoint-url http://localhost:9000 s3 ls s3://test-bucket/

test-s3-download: ## Test: Download file
	aws --endpoint-url http://localhost:9000 s3 cp s3://test-bucket/test.txt /tmp/downloaded.txt
	cat /tmp/downloaded.txt

test-s3-delete: ## Test: Delete object
	aws --endpoint-url http://localhost:9000 s3 rm s3://test-bucket/test.txt

test-s3-delete-bucket: ## Test: Delete bucket
	aws --endpoint-url http://localhost:9000 s3 rb s3://test-bucket

test-all: test-s3-create-bucket test-s3-upload test-s3-list-objects test-s3-download test-s3-delete test-s3-delete-bucket ## Run all S3 tests
	@echo "All tests passed!"

# Documentation
docs-install: ## Install documentation dependencies
	pip install -r docs/requirements.txt

docs-serve: ## Serve documentation locally
	mkdocs serve

docs-build: ## Build documentation
	mkdocs build --strict

docs-deploy: ## Deploy documentation to GitHub Pages
	mkdocs gh-deploy --force

# Release
release-patch: ## Create patch release
	@./scripts/release.sh patch

release-minor: ## Create minor release
	@./scripts/release.sh minor

release-major: ## Create major release
	@./scripts/release.sh major
