ifeq ($(shell uname -s), Darwin)
	CPU_CORES = $(shell sysctl -n hw.ncpu)
else
	CPU_CORES = $(shell grep -c processor /proc/cpuinfo)
endif

.PHONY:	help
help: ## show help message.
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY:	check
check: ## check compile is succeed
	@cargo check -j $(CPU_CORES)

.PHONY:	build
build: ## build application
	@cargo build -j $(CPU_CORES)

.PHONY:	release
release: ## build static linked binary as release using Docker
	@./script/build_release.sh

.PHONY:	run
run: ## run: cargo run
	@cargo run --quiet -j $(CPU_CORES)

.PHONY:	test
test: ## run: cargo test
	@cargo test

.PHONY:	format
format: ## run: cargo fmt
	@cargo fmt

.PHONY:	clean
clean: ## run: cargo clean
	@cargo clean
