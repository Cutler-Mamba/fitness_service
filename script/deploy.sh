#!/usr/bin/env bash
# deploy.sh - 生产环境部署
# 用法:
#   ./script/deploy.sh                    完整部署 (pull → migrate → up → health check)
#   ./script/deploy.sh --no-migrate       跳过数据库迁移
#   ./script/deploy.sh --no-pull          跳过拉取镜像 (使用本地已有镜像)
#   ./script/deploy.sh --down             停止并移除所有服务

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

NO_MIGRATE=false
NO_PULL=false
DO_DOWN=false

for arg in "$@"; do
    case $arg in
        --no-migrate) NO_MIGRATE=true ;;
        --no-pull)    NO_PULL=true ;;
        --down)       DO_DOWN=true ;;
        --help|-h)
            echo "用法: $0 [--no-migrate] [--no-pull] [--down]"
            echo "  --no-migrate  跳过数据库迁移"
            echo "  --no-pull     跳过拉取镜像"
            echo "  --down        停止并移除所有服务"
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
check_cmd docker "请安装 Docker Desktop: https://www.docker.com"

COMPOSE=$(detect_compose_cmd)
COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.yml"

# ---- 环境加载 ----
load_env
log_info "部署目标环境"

# ---- 停止服务 ----
if $DO_DOWN; then
    log_step "停止所有服务"
    $COMPOSE -f "$COMPOSE_FILE" down --remove-orphans
    log_info "服务已停止"
    exit 0
fi

# ---- 拉取镜像 ----
if ! $NO_PULL; then
    log_step "拉取最新镜像"
    if $COMPOSE -f "$COMPOSE_FILE" pull 2>/dev/null; then
        log_info "镜像拉取完成"
    else
        log_warn "镜像拉取失败，将使用本地镜像或构建"
    fi
fi

# ---- 迁移 ----
if ! $NO_MIGRATE; then
    log_step "执行数据库迁移"
    "${SCRIPT_DIR}/migrate.sh" up
    log_info "迁移完成"
else
    log_warn "跳过数据库迁移 (--no-migrate)"
fi

# ---- 启动服务 ----
log_step "启动服务"
$COMPOSE -f "$COMPOSE_FILE" up -d --remove-orphans
log_info "服务启动指令已发出"

# ---- 健康检查 ----
log_step "等待服务就绪"
"${SCRIPT_DIR}/health-check.sh"
log_info "健康检查通过"

# ---- 输出信息 ----
SERVER_HOST="${SERVER_HOST:-0.0.0.0}"
SERVER_PORT="${SERVER_PORT:-8080}"
echo ""
log_info "部署完成！"
log_info "服务地址: http://${SERVER_HOST}:${SERVER_PORT}"
log_info "查看日志: $COMPOSE -f ${COMPOSE_FILE} logs -f"
