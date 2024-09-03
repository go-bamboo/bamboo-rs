VERSION=$(shell git describe --tags --always)

.PHONY: build
build:
	RUST_BACKTRACE=1 cargo build

.PHONY: build-release
build-release:
	RUST_BACKTRACE=1 cargo build --release

.PHONY: test
test:
	cargo test --verbose

.PHONY: dev
dev: build
	RUST_BACKTRACE=1 ./target/debug/quantkline -c ./configs/dev.yaml

.PHONY: dev-test
dev-test:
	RUST_BACKTRACE=1 ./target/release/quantkline -c ./configs/test.yaml

.PHONY: migrate-dev
migrate-dev:
	goctl migrate up -c ./configs/dev.yaml

.PHONY: migrate-test
migrate-test:
	goctl migrate up -c ./configs/test.yaml

.PHONY: clean
clean:
	rm -rf ./target

.PHONY: install
install:
	cargo install sea-orm-cli

.PHONY: gen
gen:
	sea-orm-cli generate entity --ignore-tables schema_migrations -o data/src/entity  -u mysql://root:123456@localhost/quant