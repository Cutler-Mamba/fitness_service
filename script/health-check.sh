#!/usr/bin/env bash
# health-check.sh - 容器健康检查
# 用法: ./script/health-check.sh
# 退出码: 0 = 健康, 1 = 异常

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

COMPOSE=$(detect_compose_cmd)
COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.yml"
MAX_RETRY=${MAX_RETRY:-30}
RETRY_INTERVAL=${RETRY_INTERVAL:-2}

SERVICES=("fitness-app" "postgres" "redis")
ALL_HEALTHY=false

log_step "健康检查 - 容器状态"

for i in $(seq 1 $MAX_RETRY); do
    failed=0
    for svc in "${SERVICES[@]}"; do
        state=$($COMPOSE -f "$COMPOSE_FILE" ps --format json "$svc" 2>/dev/null | grep -o '"Health":"[^"]*"' | cut -d'"' -f4 || echo "missing")

        if [ "$state" = "healthy" ]; then
            printf "  ${GREEN}✓${NC} %-20s healthy\n" "$svc"
        elif [ "$state" = "starting" ]; then
            printf "  ${YELLOW}⏳${NC} %-20s starting...\n" "$svc"
            failed=$((failed + 1))
        else
            printf "  ${RED}✗${NC} %-20s %s\n" "$svc" "${state:-missing}"
            failed=$((failed + 1))
        fi
    done

    if [ $failed -eq 0 ]; then
        ALL_HEALTHY=true
        break
    fi

    if [ $i -lt $MAX_RETRY ]; then
        sleep "$RETRY_INTERVAL"
    fi

    # 回到上一行重新输出
    if [ $i -gt 1 ]; then
        tput cuu ${#SERVICES[@]} 2>/dev/null || true
    fi
done

if $ALL_HEALTHY; then
    exit 0
else
    log_error "健康检查超时！以下服务未就绪："
    for svc in "${SERVICES[@]}"; do
        log=$($COMPOSE -f "$COMPOSE_FILE" logs --tail 5 "$svc" 2>/dev/null || echo "无法获取日志")
        printf "  --- %s 最近日志 ---\n" "$svc"
        echo "$log"
    done
    exit 1
fi
