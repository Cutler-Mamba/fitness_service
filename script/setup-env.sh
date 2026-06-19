#!/usr/bin/env bash
# setup-env.sh - 初始化 .env 配置文件
# 用法: ./script/setup-env.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

ENV_FILE="${PROJECT_ROOT}/.env"
ENV_EXAMPLE="${PROJECT_ROOT}/.env.example"

log_step "AI 健身助手 - 环境初始化"

# ---- 已有 .env ----
if [ -f "$ENV_FILE" ]; then
    log_warn ".env 已存在"
    if ! confirm "是否覆盖当前 .env？"; then
        log_info "跳过，保留现有 .env"
        exit 0
    fi
    cp "$ENV_FILE" "${ENV_FILE}.bak.$(date +%Y%m%d%H%M%S)"
    log_info "已备份原 .env"
fi

# ---- 从模板复制 ----
if [ ! -f "$ENV_EXAMPLE" ]; then
    log_error ".env.example 不存在"
    exit 1
fi
cp "$ENV_EXAMPLE" "$ENV_FILE"
log_info "已从 .env.example 生成 .env"

# ---- 生成 JWT_SECRET ----
JWT_SECRET=$(openssl rand -hex 32 2>/dev/null || echo "dev-secret-$(date +%s)")
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/^JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" "$ENV_FILE"
else
    sed -i "s/^JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" "$ENV_FILE"
fi
log_info "已自动生成 JWT_SECRET"

# ---- 交互式配置 ----
echo ""
echo "请填写以下关键配置（留空保持默认）："

# LLM API Key
read -r -p "LLM API Key [${LLM_API_KEY:-}]: " input_llm_key
if [ -n "${input_llm_key:-}" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s|^LLM_API_KEY=.*|LLM_API_KEY=${input_llm_key}|" "$ENV_FILE"
    else
        sed -i "s|^LLM_API_KEY=.*|LLM_API_KEY=${input_llm_key}|" "$ENV_FILE"
    fi
fi

# LLM Model
read -r -p "LLM Model [gpt-4o]: " input_llm_model
if [ -n "${input_llm_model:-}" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^LLM_MODEL=.*/LLM_MODEL=${input_llm_model}/" "$ENV_FILE"
    else
        sed -i "s/^LLM_MODEL=.*/LLM_MODEL=${input_llm_model}/" "$ENV_FILE"
    fi
fi

# WeChat Channel
echo ""
echo "微信通道配置："
echo "  1) iLink 长轮询（推荐，无需公网IP，用于微信个人号）"
echo "  2) Webhook 回调（需要公网IP，用于微信公众号/对话开放平台）"
read -r -p "选择微信通道 [1]: " wechat_channel_choice
case "${wechat_channel_choice:-1}" in
    2)
        # Webhook mode
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' 's/^WECHAT_CHANNEL=.*/WECHAT_CHANNEL=webhook/' "$ENV_FILE"
        else
            sed -i 's/^WECHAT_CHANNEL=.*/WECHAT_CHANNEL=webhook/' "$ENV_FILE"
        fi
        read -r -p "WeChat Bot Token: " input_wc_token
        if [ -n "${input_wc_token:-}" ]; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/^# WECHAT_BOT_TOKEN=.*/WECHAT_BOT_TOKEN=${input_wc_token}/" "$ENV_FILE"
            else
                sed -i "s/^# WECHAT_BOT_TOKEN=.*/WECHAT_BOT_TOKEN=${input_wc_token}/" "$ENV_FILE"
            fi
        fi
        echo "Webhook 模式已配置，后续可在微信对话开放平台设置回调地址。"
        ;;
    *)
        # iLink mode (default)
        echo "iLink 长轮询模式已选择。"
        echo "请稍后运行以下命令扫码登录获取凭证："
        echo ""
        echo "  cargo run -p fitness-cli -- wechat login"
        echo ""
        echo "扫码成功后，将输出的 WECHAT_ACCOUNT_ID 和 WECHAT_TOKEN 填入 .env 即可。"
        ;;
esac

# Feishu
read -r -p "Feishu App ID (留空跳过): " input_fs_app_id
if [ -n "${input_fs_app_id:-}" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^FEISHU_APP_ID=.*/FEISHU_APP_ID=${input_fs_app_id}/" "$ENV_FILE"
    else
        sed -i "s/^FEISHU_APP_ID=.*/FEISHU_APP_ID=${input_fs_app_id}/" "$ENV_FILE"
    fi
    read -r -p "Feishu App Secret: " input_fs_app_secret
    if [ -n "${input_fs_app_secret:-}" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^FEISHU_APP_SECRET=.*/FEISHU_APP_SECRET=${input_fs_app_secret}/" "$ENV_FILE"
        else
            sed -i "s/^FEISHU_APP_SECRET=.*/FEISHU_APP_SECRET=${input_fs_app_secret}/" "$ENV_FILE"
        fi
    fi
fi

echo ""
log_info ".env 配置完成！"
log_info "如需修改，直接编辑: ${ENV_FILE}"
