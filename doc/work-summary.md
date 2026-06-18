# AI 健身助手 — 开发工作总览

> 最后更新: 2026-06-18

---

## 1. 项目概述

**AI 健身助手** 是一款通过 LLM 驱动的对话与生成能力，为用户提供个性化训练计划、营养建议、动作指导等服务的智能健身助手。

### MVP 产品决策

| 维度 | 决策 |
|---|---|
| 用户入口 | **微信个人号**（iLink Bot API，官方合规接口） |
| 租户模型 | 租户 = 微信用户，管理员 CLI 手动开通 |
| MVP 功能 | AI 对话、训练计划生成、营养建议 |
| 免费期限流 | 不做系统硬限制，人工管理 |
| API Key | 服务端统一配置，用户无需提供 |

---

## 2. 技术栈

| 组件 | 选择 |
|---|---|
| 语言 | Rust Edition 2024 |
| Web 框架 | Axum 0.8 |
| ORM | SeaORM 1.x |
| 数据库 | PostgreSQL（生产）/ SQLite（开发） |
| 缓存 | Redis |
| LLM | async-openai 0.27（兼容 OpenAI 协议） |
| 微信通道 | iLink Bot API（`ilinkai.weixin.qq.com`） |
| 认证 | JWT + Argon2 |
| 部署 | Docker Compose（app + postgres + redis） |

---

## 3. 项目结构

```
fitness_service/
├── Cargo.toml                 # Workspace 配置
├── Cargo.lock
├── Dockerfile                 # 多阶段构建
├── docker-compose.yml         # 3 服务编排
├── .env.example               # 环境变量模板
├── .gitignore
├── README.md
├── LICENSE
│
├── design/
│   └── product-architecture.md   # 早期设计文档（飞书版）
│
├── doc/                       # 文档目录（本文档所在）
├── tests/                     # 集成测试（待实现）
│
├── config/
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
│
├── script/
│   ├── lib/common.sh          # 公共函数库
│   ├── build.sh               # 构建脚本（支持跨平台）
│   ├── deploy.sh              # 生产部署
│   ├── migrate.sh             # 数据库迁移管理
│   ├── health-check.sh        # 健康检查
│   └── setup-env.sh           # 交互式 .env 生成
│
├── crates/
│   ├── fitness-core/          # 共享核心库
│   │   └── src/
│   │       ├── config.rs      # 配置（含 WechatConfig）
│   │       ├── error.rs       # 统一错误类型 AppError
│   │       ├── types.rs       # 共享类型、枚举、数据结构
│   │       └── lib.rs
│   │
│   ├── fitness-entity/        # 数据模型（9 表）
│   │   └── src/
│   │       ├── user.rs
│   │       ├── tenant.rs
│   │       ├── workout_plan.rs
│   │       ├── exercise.rs
│   │       ├── workout_log.rs
│   │       ├── nutrition_log.rs
│   │       ├── body_metrics.rs
│   │       ├── chat_session.rs
│   │       ├── chat_message.rs
│   │       └── lib.rs
│   │
│   ├── fitness-migration/     # 数据库迁移（9 迁移）
│   │   └── src/
│   │       ├── lib.rs         # 所有 migration 定义
│   │       └── main.rs        # CLI 入口
│   │
│   ├── fitness-llm/           # LLM 集成层
│   │   └── src/
│   │       ├── client.rs      # LlmClient（async-openai 封装）
│   │       ├── schema.rs      # 结构化输出 Schema
│   │       ├── prompt/        # Prompt 模板
│   │       │   ├── mod.rs
│   │       │   ├── chat.rs            # 通用对话
│   │       │   ├── plan_generation.rs # 训练计划
│   │       │   ├── nutrition_advice.rs # 营养建议
│   │       │   └── form_check.rs      # 动作纠正
│   │       └── lib.rs
│   │
│   ├── fitness-service/       # 业务逻辑层
│   │   └── src/
│   │       ├── user_service.rs      # ✅ 完整
│   │       ├── tenant_service.rs    # ✅ 完整
│   │       ├── ai_service.rs        # ✅ 完整
│   │       ├── plan_service.rs      # 🔧 预留
│   │       ├── exercise_service.rs  # 🔧 预留
│   │       ├── nutrition_service.rs # 🔧 预留
│   │       ├── metrics_service.rs   # 🔧 预留
│   │       └── lib.rs
│   │
│   ├── fitness-handler/       # HTTP API 层
│   │   └── src/
│   │       ├── auth_handler.rs      # 注册/登录
│   │       ├── user_handler.rs      # 用户个人信息
│   │       ├── ai_handler.rs        # AI 对话/计划/营养
│   │       ├── stub_handler.rs      # 预留路由
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   └── auth.rs          # JWT 提取
│   │       └── lib.rs               # ApiState + 路由组装
│   │
│   ├── fitness-bot/           # 微信机器人
│   │   └── src/
│   │       ├── handler.rs     # Webhook 事件处理
│   │       ├── command.rs     # 命令解析器
│   │       └── lib.rs
│   │
│   ├── fitness-app/           # 主二进制入口
│   │   └── src/
│   │       ├── main.rs        # 服务启动 + 路由
│   │       └── state.rs       # AppState 初始化
│   │
│   └── fitness-cli/           # 管理 CLU 工具
│       └── src/
│           └── main.rs        # 租户管理命令
```

---

## 4. 数据库 Schema

### 4.1 `users` 表（传统账号体系 + 飞书绑定）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | 主键 |
| `phone` | VARCHAR (unique) | 手机号 |
| `email` | VARCHAR (unique) | 邮箱 |
| `password_hash` | VARCHAR | Argon2 哈希 |
| `nickname` | VARCHAR NOT NULL | 昵称 |
| `avatar` | VARCHAR | 头像 URL |
| `fitness_level` | VARCHAR | 健身水平 |
| `gender` | VARCHAR | 性别 |
| `birth_date` | DATE | 出生日期 |
| `height` | DECIMAL | 身高 |
| `weight` | DECIMAL | 体重 |
| `feishu_open_id` | VARCHAR (unique) | 飞书 OpenID |
| `created_at` | TIMESTAMPTZ | 创建时间 |
| `updated_at` | TIMESTAMPTZ | 更新时间 |

### 4.2 `tenants` 表（微信租户）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | 主键 |
| `wechat_user_id` | VARCHAR (unique) NOT NULL | iLink 微信用户标识 |
| `nickname` | VARCHAR | 微信昵称 |
| `status` | VARCHAR NOT NULL, default `disabled` | `active` / `disabled` |
| `daily_quota` | INT (nullable) | 每日配额（NULL = 不限） |
| `created_at` | TIMESTAMPTZ | 创建时间 |
| `updated_at` | TIMESTAMPTZ | 更新时间 |

### 4.3 `workout_plans` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `user_id` | UUID NOT NULL | 关联用户 |
| `name` | VARCHAR NOT NULL | 计划名称 |
| `goal` | VARCHAR NOT NULL | 目标 |
| `difficulty` | VARCHAR | 难度 |
| `duration_weeks` | INT NOT NULL | 周数 |
| `schedule` | JSON | 课表（JSON） |
| `tips` | JSON | 建议列表 |
| `status` | VARCHAR NOT NULL | active/completed/paused |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

### 4.4 `exercises` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `plan_id` | UUID | 关联计划 |
| `name` | VARCHAR NOT NULL | 动作名称 |
| `sets` | INT NOT NULL | 组数 |
| `reps` | VARCHAR NOT NULL | 次数（如 "8-12"） |
| `rest_seconds` | INT NOT NULL | 组间休息 |
| `notes` | VARCHAR | 注意事项 |
| `order_index` | INT NOT NULL | 排序 |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

### 4.5 `workout_logs` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `user_id` | UUID NOT NULL | |
| `plan_id` | UUID | |
| `exercise_name` | VARCHAR NOT NULL | |
| `sets_completed` | INT | |
| `reps_completed` | VARCHAR | |
| `weight_kg` | DECIMAL | |
| `duration_seconds` | INT | |
| `notes` | VARCHAR | |
| `logged_at` | TIMESTAMPTZ | |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

### 4.6 `nutrition_logs` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `user_id` | UUID NOT NULL | |
| `meal_type` | VARCHAR NOT NULL | 餐次类型 |
| `food_name` | VARCHAR NOT NULL | |
| `amount` | DECIMAL | |
| `unit` | VARCHAR | |
| `calories` | DECIMAL | |
| `protein_g` | DECIMAL | |
| `carbs_g` | DECIMAL | |
| `fat_g` | DECIMAL | |
| `notes` | VARCHAR | |
| `logged_at` | TIMESTAMPTZ | |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

### 4.7 `body_metrics` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `user_id` | UUID NOT NULL | |
| `weight_kg` | DECIMAL | |
| `body_fat_pct` | DECIMAL | |
| `muscle_mass_kg` | DECIMAL | |
| `waist_cm` | DECIMAL | |
| `hip_cm` | DECIMAL | |
| `chest_cm` | DECIMAL | |
| `notes` | VARCHAR | |
| `measured_at` | TIMESTAMPTZ | |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

### 4.8 `chat_sessions` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `user_id` | UUID NOT NULL | |
| `title` | VARCHAR | 会话标题 |
| `created_at` | TIMESTAMPTZ | |
| `updated_at` | TIMESTAMPTZ | |

### 4.9 `chat_messages` 表（预留）

| 列名 | 类型 | 说明 |
|---|---|---|
| `id` | UUID PK | |
| `session_id` | UUID NOT NULL | |
| `role` | VARCHAR NOT NULL | user/assistant/system |
| `content` | VARCHAR NOT NULL | |
| `token_count` | INT | Token 消耗 |
| `created_at` | TIMESTAMPTZ | |

---

## 5. API 端点

### 5.1 认证 (`/api/v1/auth`) ✅

| 方法 | 路径 | 说明 | 鉴权 |
|---|---|---|---|
| POST | `/api/v1/auth/register` | 邮箱/手机注册 | 无 |
| POST | `/api/v1/auth/login` | 登录获取 JWT | 无 |

### 5.2 用户 (`/api/v1/users`) ✅

| 方法 | 路径 | 说明 | 鉴权 |
|---|---|---|---|
| GET | `/api/v1/users/me` | 获取当前用户信息 | JWT |
| PUT | `/api/v1/users/me` | 更新个人信息 | JWT |

### 5.3 AI (`/api/v1/ai`) ✅

| 方法 | 路径 | 说明 | 鉴权 |
|---|---|---|---|
| POST | `/api/v1/ai/chat` | 通用健身 AI 对话 | X-Wechat-User-Id |
| POST | `/api/v1/ai/plan` | 生成训练计划 | X-Wechat-User-Id |
| POST | `/api/v1/ai/nutrition` | 营养分析 | X-Wechat-User-Id |

请求示例：
```json
// POST /api/v1/ai/chat
{ "message": "我想减脂，每周能练3天", "profile": { "goal": "lose_weight", "weekly_days_available": 3 } }

// POST /api/v1/ai/plan
{ "profile": { "level": "beginner", "goal": "lose_weight", "height_cm": 170, "weight_kg": 80 } }

// POST /api/v1/ai/nutrition
{ "food_input": "今天吃了：早餐燕麦牛奶，午餐鸡胸沙拉，晚餐米饭红烧肉", "profile": { "goal": "lose_weight" } }
```

### 5.4 微信 Webhook (`/api/v1/wechat`) ✅

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/v1/wechat/event` | iLink Bot 消息回调和事件 |
| POST | `/api/v1/wechat/challenge` | URL 验证 |

### 5.5 预留路由 🔧

| 前缀 | 说明 |
|---|---|
| `/api/v1/plans` | 训练计划管理 |
| `/api/v1/exercises` | 动作库管理 |
| `/api/v1/nutrition` | 营养记录管理 |
| `/api/v1/metrics` | 身体数据管理 |

---

## 6. 服务层详情

### 6.1 `UserService` ✅

| 方法 | 说明 |
|---|---|
| `register()` | 邮箱/手机 + 密码注册，Argon2 哈希 |
| `login()` | 邮箱或手机 + 密码登录 |
| `find_by_id()` | 按 UUID 查询用户 |
| `find_by_feishu_open_id()` | 按飞书 OpenID 查找 |
| `update_profile()` | 更新用户健身档案 |
| `bind_feishu()` | 绑定飞书账号 |

### 6.2 `TenantService` ✅

| 方法 | 说明 |
|---|---|
| `find_by_wechat_id()` | 按微信用户 ID 查找租户 |
| `is_active()` | 检查租户是否激活 |
| `create()` | 创建租户（默认 active） |
| `list_all()` | 列出所有租户 |
| `set_status()` | 设置租户状态（active/disabled） |
| `find_or_create()` | 首次对话时自动创建记录（status=disabled） |

### 6.3 `AiService` ✅

| 方法 | 说明 | 输出类型 |
|---|---|---|
| `chat()` | 通用健身 AI 对话 | 文本 |
| `generate_plan()` | 生成结构化训练计划 | `GeneratedPlanOutput` (JSON) |
| `analyze_nutrition()` | 分析饮食并给出营养建议 | `NutritionAnalysisOutput` (JSON) |

### 6.4 预留服务 🔧

| 服务 | 文件 | 状态 |
|---|---|---|
| `PlanService` | `plan_service.rs` | 壳（struct + new） |
| `ExerciseService` | `exercise_service.rs` | 壳（struct + new） |
| `NutritionService` | `nutrition_service.rs` | 壳（struct + new） |
| `MetricsService` | `metrics_service.rs` | 壳（struct + new） |

---

## 7. LLM 集成

### 7.1 `LlmClient`

封装 `async-openai`，支持两种调用模式：

| 方法 | 说明 |
|---|---|
| `chat()` | 纯文本对话 |
| `chat_with_json_schema()` | 结构化 JSON 输出（带 Schema 约束） |

### 7.2 Prompt 模板

| 模板 | 用途 | 输出格式 |
|---|---|---|
| `chat()` | 通用健身助手对话 | 自由文本 |
| `plan_generation()` | 训练计划生成 | JSON（GeneratedPlanOutput） |
| `nutrition_advice()` | 营养分析和建议 | JSON（NutritionAnalysisOutput） |
| `form_check()` | 动作纠正指导 | 自由文本 |

### 7.3 结构化输出 Schema

```rust
// 训练计划
GeneratedPlanOutput {
    name, goal, difficulty, duration_weeks,
    schedule: { "周一": ["深蹲", "卧推"], ... },
    exercises: [{ name, sets, reps, rest_seconds, notes }],
    tips: [...]
}

// 营养分析
NutritionAnalysisOutput {
    total_calories, protein_g, carbs_g, fat_g,
    assessment, suggestions: [...]
}
```

---

## 8. 微信机器人

### 8.1 命令解析

| 命令 | 说明 |
|---|---|
| `/plan [目标]` | 生成训练计划 |
| `/log <内容>` | 记录训练 |
| `/diet [要求]` | 饮食建议 |
| `/stats` | 查看统计 |
| `/help` | 帮助 |
| 其他文本 | AI 自由对话 |

### 8.2 鉴权流程

```
微信用户发消息 → Webhook 收到事件
  → 提取 X-Wechat-User-Id
  → 查 tenants 表：
    ├─ 不存在 → 自动创建（status=disabled），返回"请等待管理员开通"
    ├─ 存在但 disabled → 返回"服务未开通"
    └─ active → 正常处理
```

---

## 9. CLI 管理工具

### 编译

```bash
cargo run -p fitness-cli -- <command>
# 或直接运行二进制
./fitness-cli <command>
```

### 命令

```bash
# 创建租户
fitness-cli tenant create <WECHAT_USER_ID> [--nickname NAME]

# 列出所有租户
fitness-cli tenant list

# 设置租户状态
fitness-cli tenant status <WECHAT_USER_ID> active|disabled

# 查看租户详情
fitness-cli tenant show <WECHAT_USER_ID>
```

---

## 10. 构建 & 部署

### 10.1 本地开发

```bash
# Debug 构建
./script/build.sh

# Release 构建 + 测试 + Lint
./script/build.sh --release --test --lint
```

### 10.2 macOS → Linux 跨平台打包

```bash
# Linux 二进制（动态链接 glibc）
./script/build.sh --release --target linux
# 输出: dist/linux/fitness-app, fitness-cli, fitness-migrate

# Linux 静态二进制（musl，无依赖）
./script/build.sh --release --target linux-musl
# 输出: dist/linux-musl/fitness-app, fitness-cli, fitness-migrate
```

### 10.3 Docker 部署

```bash
# 完整部署（含迁移 + 健康检查）
./script/deploy.sh

# 跳过迁移
./script/deploy.sh --no-migrate

# 停止服务
./script/deploy.sh --down
```

### 10.4 数据库迁移

```bash
./script/migrate.sh up       # 执行迁移
./script/migrate.sh down     # 回滚
./script/migrate.sh status   # 查看状态
./script/migrate.sh fresh    # 重置并重新迁移
```

---

## 11. 配置项

完整环境变量见 `.env.example`：

| 变量 | 默认值 | 说明 |
|---|---|---|
| `DATABASE_URL` | `sqlite:./data/fitness.db?mode=rwc` | 数据库连接 |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis 连接 |
| `JWT_SECRET` | `dev-secret-change-in-production` | JWT 签名密钥 |
| `LLM_API_KEY` | — | LLM API Key |
| `LLM_MODEL` | `gpt-4o` | 模型名称 |
| `WECHAT_ILINK_URL` | `https://ilinkai.weixin.qq.com` | iLink API 地址 |
| `WECHAT_BOT_TOKEN` | — | Bot Token |
| `WECHAT_VERIFICATION_TOKEN` | — | Webhook 验证 Token |
| `SERVER_PORT` | `8080` | 服务端口 |

---

## 12. 完成度总览

| 模块 | 状态 | 说明 |
|---|---|---|
| 项目骨架 | ✅ 完成 | 9 crate workspace，依赖管理 |
| 配置系统 | ✅ 完成 | 环境变量 + WeChat/Fastmail 双通道 |
| 错误处理 | ✅ 完成 | `AppError` 10 变体 + `IntoResponse` |
| 数据层 | ✅ 完成 | 9 表 Entity + Migration |
| 用户服务 | ✅ 完成 | 注册/登录/个人信息/JWT |
| 租户服务 | ✅ 完成 | CRUD + 状态管理 + 自动注册 |
| LLM 集成 | ✅ 完成 | 4 Prompt + 2 Schema + 结构化输出 |
| AI 服务 | ✅ 完成 | chat/plan/nutrition |
| AI API | ✅ 完成 | 3 endpoint + 租户鉴权 |
| 微信 Bot | ✅ 完成 | Webhook + 命令解析 |
| CLI 工具 | ✅ 完成 | 4 租户管理命令 |
| 构建脚本 | ✅ 完成 | 跨平台构建 + Docker |
| 部署脚本 | ✅ 完成 | Docker Compose + 迁移 |
| 预留服务/路由 | ✅ 完成 | 4 服务壳 + 4 路由 |
| 飞书支持 | ⚠️ 保留 | 代码保留，路由已替换为微信 |
| 训练记录 | 🔧 预留 | DB + Service + Handler 壳就绪 |
| 营养记录 | 🔧 预留 | DB + Service + Handler 壳就绪 |
| 身体数据 | 🔧 预留 | DB + Service + Handler 壳就绪 |
| 动作纠正 | 🔧 预留 | Prompt 已就绪 |
| 订阅付费 | ❌ 未开始 | |
| 限流中间件 | ❌ 未开始 | |
| SSE 流式输出 | ❌ 未开始 | |
| Web 前端 | ❌ 未开始 | |
| macOS 桌面端 | ❌ 未开始 | |
| 集成测试 | ❌ 未开始 | `tests/` 目录为空 |

### 统计

- **总提交**: 4 commits
- **Crate 数量**: 9
- **Rust 源文件**: ~30 个
- **数据库表**: 9（2 在用 + 7 预留）
- **API 端点**: 7 在用 + 4 预留
- **二进制产物**: 3（fitness-app、fitness-cli、fitness-migrate）
