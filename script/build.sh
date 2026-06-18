#!/usr/bin/env bash
# build.sh - 构建脚本（支持 macOS → Linux 跨平台）
# 用法:
#   ./script/build.sh                      默认: fmt check + debug build
#   ./script/build.sh --release            release build (当前平台)
#   ./script/build.sh --release --target linux   构建 Linux 二进制 (via Docker)
#   ./script/build.sh --release --target linux-musl  构建 Linux 静态链接二进制
#   ./script/build.sh --test               同时跑测试
#   ./script/build.sh --lint               同时跑 clippy

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

RELEASE=false
RUN_TEST=false
RUN_LINT=false
CROSS_TARGET=""
DOCKER=false

for arg in "$@"; do
    case $arg in
        --release) RELEASE=true ;;
        --test)    RUN_TEST=true ;;
        --lint)    RUN_LINT=true ;;
        --target)
            shift
            CROSS_TARGET="${1:-}"
            DOCKER=true
            ;;
        --help|-h)
            echo "用法: $0 [FLAGS]"
            echo ""
            echo "本地构建:"
            echo "  --release        release build (当前平台)"
            echo "  --test           构建并运行测试"
            echo "  --lint           构建并运行 clippy"
            echo ""
            echo "Linux 跨平台构建 (via Docker):"
            echo "  --target linux       构建 x86_64 Linux 二进制（动态链接）"
            echo "  --target linux-musl  构建 x86_64 Linux 静态链接二进制"
            echo ""
            echo "示例:"
            echo "  $0                                    # debug build"
            echo "  $0 --release                          # release build"
            echo "  $0 --release --lint --test            # 完整检查"
            echo "  $0 --release --target linux           # macOS → Linux 二进制"
            echo "  $0 --release --target linux-musl      # macOS → Linux 静态二进制"
            exit 0
            ;;
        *)
            log_error "未知参数: $arg"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# ---- 工具检查 ----
check_cmd cargo "curl https://sh.rustup.rs -sSf | sh"

# ---- Docker 跨平台构建 ----
if $DOCKER; then
    check_cmd docker "请安装 Docker Desktop: https://www.docker.com"

    case "$CROSS_TARGET" in
        linux)
            log_step "Docker 构建 Linux x86_64 二进制（动态链接 glibc）"
            docker build -t fitness-builder -f - . <<'DOCKERFILE'
FROM rust:1.93-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/
RUN mkdir -p /app/output && \
    cargo build --release --bin fitness-app --bin fitness-cli --bin fitness-migrate && \
    cp target/release/fitness-app /app/output/ && \
    cp target/release/fitness-cli /app/output/ && \
    cp target/release/fitness-migrate /app/output/
FROM scratch
COPY --from=builder /app/output/ /output/
DOCKERFILE
            CONTAINER_ID=$(docker create fitness-builder)
            mkdir -p "${PROJECT_ROOT}/dist/linux"
            docker cp "$CONTAINER_ID":/output/. "${PROJECT_ROOT}/dist/linux/"
            docker rm "$CONTAINER_ID"
            docker rmi fitness-builder 2>/dev/null || true
            ;;

        linux-musl)
            log_step "Docker 构建 Linux x86_64 静态链接二进制 (musl)"

            local rust_ver="1.93"
            docker build -t fitness-builder-musl -f - . <<DOCKERFILE
FROM rust:${rust_ver}-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/
RUN mkdir -p /app/output && \
    cargo build --release --bin fitness-app --bin fitness-cli --bin fitness-migrate && \
    cp target/release/fitness-app /app/output/ && \
    cp target/release/fitness-cli /app/output/ && \
    cp target/release/fitness-migrate /app/output/
FROM scratch
COPY --from=builder /app/output/ /output/
DOCKERFILE
            CONTAINER_ID=$(docker create fitness-builder-musl)
            mkdir -p "${PROJECT_ROOT}/dist/linux-musl"
            docker cp "$CONTAINER_ID":/output/. "${PROJECT_ROOT}/dist/linux-musl/"
            docker rm "$CONTAINER_ID"
            docker rmi fitness-builder-musl 2>/dev/null || true
            ;;

        *)
            log_error "未知 target: ${CROSS_TARGET} (支持: linux, linux-musl)"
            exit 1
            ;;
    esac

    OUTPUT_DIR="${PROJECT_ROOT}/dist/${CROSS_TARGET}"
    log_info "构建完成！输出目录: ${OUTPUT_DIR}"
    ls -lh "${OUTPUT_DIR}/fitness-app" "${OUTPUT_DIR}/fitness-cli" "${OUTPUT_DIR}/fitness-migrate" 2>/dev/null || true
    exit 0
fi

# ---- 格式化检查 ----
log_step "检查代码格式 (cargo fmt --check)"
cargo fmt --check
log_info "格式检查通过"

# ---- Clippy ----
if $RUN_LINT; then
    log_step "运行 clippy"
    cargo clippy --all-targets
    log_info "clippy 通过"
fi

# ---- 构建 ----
if $RELEASE; then
    log_step "Release 构建 (cargo build --release)"
    cargo build --release
    BIN_DIR="${PROJECT_ROOT}/target/release"
else
    log_step "Debug 构建 (cargo build)"
    cargo build
    BIN_DIR="${PROJECT_ROOT}/target/debug"
fi
log_info "构建成功"
log_info "输出目录: ${BIN_DIR}"

# ---- 测试 ----
if $RUN_TEST; then
    log_step "运行测试 (cargo test)"
    cargo test
    log_info "测试通过"
fi
