#!/usr/bin/env bash
# common.sh - 公共函数库
# 被其他脚本 source 引入

set -euo pipefail

# ---- 颜色 ----
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# ---- 日志 ----
log_info()  { printf "${GREEN}[INFO]${NC}  %s\n" "$*"; }
log_warn()  { printf "${YELLOW}[WARN]${NC}  %s\n" "$*"; }
log_error() { printf "${RED}[ERROR]${NC} %s\n" "$*"; }
log_step()  { printf "\n${CYAN}==>${NC} %s\n" "$*"; }

# ---- 路径 ----
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ---- 工具检查 ----
check_cmd() {
    local cmd=$1
    local hint=${2:-}
    if ! command -v "$cmd" &>/dev/null; then
        log_error "缺少命令: ${cmd}"
        if [ -n "$hint" ]; then
            log_info "安装提示: ${hint}"
        fi
        exit 1
    fi
}

# ---- 环境加载 ----
load_env() {
    local env_file="${PROJECT_ROOT}/.env"
    if [ -f "$env_file" ]; then
        set -a
        source "$env_file"
        set +a
    else
        log_warn ".env 文件不存在，请先运行: ./script/setup-env.sh"
    fi
}

# ---- Docker Compose 检测 ----
detect_compose_cmd() {
    if docker compose version &>/dev/null 2>&1; then
        echo "docker compose"
    elif docker-compose version &>/dev/null 2>&1; then
        echo "docker-compose"
    else
        log_error "未找到 docker compose 或 docker-compose"
        exit 1
    fi
}

# ---- 确认操作 ----
confirm() {
    local prompt=${1:-"确认继续?"}
    read -r -p "$prompt [y/N] " response
    case "$response" in
        [yY][eE][sS]|[yY]) return 0 ;;
        *) return 1 ;;
    esac
}
