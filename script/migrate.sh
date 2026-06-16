#!/usr/bin/env bash
# migrate.sh - 数据库迁移管理
# 用法:
#   ./script/migrate.sh up       执行迁移
#   ./script/migrate.sh down     回滚最近一次迁移
#   ./script/migrate.sh status   查看迁移状态
#   ./script/migrate.sh fresh    重置数据库并重新迁移

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

ACTION="${1:-status}"
COMPOSE=$(detect_compose_cmd)

# ---- 环境加载 ----
load_env

# ---- 检查容器是否运行 ----
check_services() {
    local running
    running=$($COMPOSE -f "${PROJECT_ROOT}/docker-compose.yml" ps --status running -q 2>/dev/null || true)
    if [ -z "$running" ]; then
        log_error "Docker 服务未运行，请先执行: ./script/deploy.sh"
        exit 1
    fi
}

# ---- 用 sea-orm-cli 在容器中执行迁移 ----
run_migration() {
    local action=$1
    local db_url="${DATABASE_URL:-sqlite:./data/fitness.db?mode=rwc}"

    log_info "目标数据库: ${db_url}"

    case $action in
        up)
            log_step "执行迁移 (up)"
            $COMPOSE -f "${PROJECT_ROOT}/docker-compose.yml" \
                run --rm -e "DATABASE_URL=${db_url}" \
                fitness-app sh -c "sea-orm-cli migrate up" \
                2>/dev/null || {
                    log_warn "sea-orm-cli 不可用，尝试 cargo run"
                    cd "$PROJECT_ROOT" && cargo run -p fitness-migration -- up
                }
            ;;
        down)
            log_step "回滚迁移 (down)"
            $COMPOSE -f "${PROJECT_ROOT}/docker-compose.yml" \
                run --rm -e "DATABASE_URL=${db_url}" \
                fitness-app sh -c "sea-orm-cli migrate down" \
                2>/dev/null || {
                    log_warn "sea-orm-cli 不可用，尝试 cargo run"
                    cd "$PROJECT_ROOT" && cargo run -p fitness-migration -- down
                }
            ;;
        status)
            log_step "迁移状态"
            $COMPOSE -f "${PROJECT_ROOT}/docker-compose.yml" \
                run --rm -e "DATABASE_URL=${db_url}" \
                fitness-app sh -c "sea-orm-cli migrate status" \
                2>/dev/null || {
                    log_warn "sea-orm-cli 不可用，尝试 cargo run"
                    cd "$PROJECT_ROOT" && cargo run -p fitness-migration -- status
                }
            ;;
        fresh)
            log_warn "即将清空所有数据！"
            if confirm "确认执行 fresh（清空数据库并重新迁移）？"; then
                $COMPOSE -f "${PROJECT_ROOT}/docker-compose.yml" \
                    run --rm -e "DATABASE_URL=${db_url}" \
                    fitness-app sh -c "sea-orm-cli migrate fresh" \
                    2>/dev/null || {
                        log_warn "sea-orm-cli 不可用，尝试 cargo run"
                        cd "$PROJECT_ROOT" && cargo run -p fitness-migration -- fresh
                    }
            else
                log_info "已取消"
            fi
            ;;
        *)
            log_error "未知操作: $action (支持: up, down, status, fresh)"
            exit 1
            ;;
    esac
}

case $ACTION in
    up|down|status|fresh)
        check_services
        run_migration "$ACTION"
        ;;
    *)
        log_error "未知操作: $ACTION (支持: up, down, status, fresh)"
        exit 1
        ;;
esac
