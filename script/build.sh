#!/usr/bin/env bash
# build.sh - 本地开发构建
# 用法:
#   ./script/build.sh               默认: fmt check + build
#   ./script/build.sh --release     release build
#   ./script/build.sh --test        同时跑测试
#   ./script/build.sh --lint        同时跑 clippy

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

RELEASE=false
RUN_TEST=false
RUN_LINT=false

for arg in "$@"; do
    case $arg in
        --release) RELEASE=true ;;
        --test)    RUN_TEST=true ;;
        --lint)    RUN_LINT=true ;;
        --help|-h)
            echo "用法: $0 [--release] [--test] [--lint]"
            echo "  --release   release 构建"
            echo "  --test      构建并运行测试"
            echo "  --lint      构建并运行 clippy"
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

# ---- 格式化检查 ----
log_step "检查代码格式 (cargo fmt --check)"
cargo fmt --check
log_info "格式检查通过"

# ---- Clippy ----
if $RUN_LINT; then
    log_step "运行 clippy"
    cargo clippy --all-targets 2>&1
    log_info "clippy 通过"
fi

# ---- 构建 ----
if $RELEASE; then
    log_step "Release 构建 (cargo build --release)"
    cargo build --release
else
    log_step "Debug 构建 (cargo build)"
    cargo build
fi
log_info "构建成功"

# ---- 测试 ----
if $RUN_TEST; then
    log_step "运行测试 (cargo test)"
    cargo test
    log_info "测试通过"
fi
