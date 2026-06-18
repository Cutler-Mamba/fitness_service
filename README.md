# AI 健身助手

基于 Rust + LLM 的全方位 AI 健身助手后端服务。通过 iLink Bot 长轮询接入微信个人号，**无需公网 IP**。同时保留 Webhook 模式支持飞书、微信对话开放平台等多端接入。

## 核心功能

| 场景 | 说明 | 状态 |
|------|------|------|
| 训练计划制定 | AI 生成个性化训练计划 | P0 |
| AI 自由对话 | 健身知识问答、交互入口 | P0 |
| 微信机器人 | iLink 长轮询 / 指令系统 / AI 对话 | P0 |
| 训练记录 | 记录完成情况 | P1 |
| 饮食营养管理 | AI 分析 + 食谱推荐 | P1 |
| 身体数据追踪 | 指标记录与趋势 | P1 |
| 动作指导纠正 | 文字指导 → 视觉识别 | P2 |
| Web 前端 | SPA 应用 | 后续迭代 |
| macOS Desktop | Tauri 桌面应用 | 未来规划 |

## 技术栈

| 组件 | 选型 |
|------|------|
| 语言 | Rust (Edition 2024) |
| Web 框架 | Axum 0.8 |
| ORM | SeaORM 1.x |
| 数据库 | PostgreSQL (开发 SQLite) |
| 缓存 | Redis |
| LLM | async-openai (OpenAI/Claude) |
| 微信通道 | iLink Bot API (HTTP 长轮询, 无需公网IP) |
| 认证 | JWT + Argon2 |
| 部署 | Docker Compose |

## 项目结构

```
fitness_service/
├── crates/
│   ├── fitness-core/          # 共享类型、错误、配置
│   ├── fitness-entity/        # SeaORM 数据模型
│   ├── fitness-migration/     # 数据库迁移 + CLI
│   ├── fitness-llm/           # LLM 集成 + Prompt 模板
│   ├── fitness-service/       # 业务逻辑层
│   ├── fitness-handler/       # Axum API 路由 + 认证
│   ├── fitness-bot/           # 多通道消息机器人 (iLink长轮询/Webhook)
│   │   ├── engine.rs          # BotEngine 平台无关编排
│   │   ├── platform/          # MessagingPlatform trait + 各平台实现
│   │   └── ilink/             # iLink HTTP 客户端 / session / types
│   └── fitness-app/           # 主入口
├── config/
├── script/
├── design/
├── Dockerfile
├── docker-compose.yml
└── Cargo.toml
```

## 快速开始

### 1. 环境准备

```bash
# Rust 1.93+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Docker Desktop (可选，本地开发不需要)
brew install --cask docker   # macOS
```

### 2. 初始化配置

```bash
cp .env.example .env
# 编辑 .env，至少填写 LLM_API_KEY
# iLink 模式: 设置 WECHAT_CHANNEL=ilink, WECHAT_ACCOUNT_ID, WECHAT_TOKEN
# Webhook 模式: 设置 WECHAT_CHANNEL=webhook, WECHAT_BOT_TOKEN, WECHAT_VERIFICATION_TOKEN
```

### 3. 本地开发

```bash
# 构建
./script/build.sh

# 构建 + 测试
./script/build.sh --test

# 运行
cargo run

# 或指定日志级别
RUST_LOG=debug cargo run
```

服务默认监听 `http://0.0.0.0:8080`，开发模式默认使用 SQLite。

> **微信接入**: iLink 模式通过 HTTP 长轮询与 `ilinkai.weixin.qq.com` 通信，**无需公网 IP**，在内网 Ubuntu 服务器上即可运行。

### 4. Docker 部署

```bash
# 一键部署
./script/deploy.sh

# 查看日志
docker compose -f docker-compose.yml logs -f

# 停止服务
./script/deploy.sh --down
```

### 5. 数据库迁移

```bash
# 执行迁移
./script/migrate.sh up

# 查看状态
./script/migrate.sh status

# 回滚
./script/migrate.sh down
```

## API 概览

### 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/auth/register` | 注册 |
| POST | `/api/v1/auth/login` | 登录 |

### 用户

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/users/me` | 获取档案 |
| PUT | `/api/v1/users/me` | 更新档案 |

### AI (微信用户鉴权)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/ai/chat` | AI 对话 |
| POST | `/api/v1/ai/plan` | 生成训练计划 |
| POST | `/api/v1/ai/nutrition` | 营养分析 |

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SERVER_HOST` | 监听地址 | `0.0.0.0` |
| `SERVER_PORT` | 监听端口 | `8080` |
| `DATABASE_URL` | 数据库连接 | SQLite |
| `REDIS_URL` | Redis 连接 | `redis://127.0.0.1:6379` |
| `JWT_SECRET` | JWT 密钥 | (自动生成) |
| `LLM_API_KEY` | LLM API Key | - |
| `LLM_MODEL` | 模型名称 | `gpt-4o` |
| `WECHAT_CHANNEL` | 微信通道: `ilink` 或 `webhook` | `ilink` |
| `WECHAT_ILINK_URL` | iLink API 地址 | `https://ilinkai.weixin.qq.com` |
| `WECHAT_ACCOUNT_ID` | iLink 账号 ID | - |
| `WECHAT_TOKEN` | iLink Token | -` |

完整配置见 `.env.example`。

## 微信指令

| 指令 | 功能 |
|------|------|
| `/plan <目标>` | AI 生成训练计划 |
| `/log <内容>` | 记录训练 |
| `/diet <需求>` | 饮食建议 |
| `/stats` | 查看统计 |
| `/help` | 帮助信息 |

## 文档

- [开发工作总览](doc/work-summary.md)
- [产品架构设计](design/product-architecture.md)

## License

MIT
