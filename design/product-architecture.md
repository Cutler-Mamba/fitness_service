# AI 健身助手 - 产品架构设计

## 1. 产品概述

AI 健身助手是一款覆盖全方位健身场景的智能助手产品，通过 LLM 驱动的对话与生成能力，为用户提供个性化的训练计划、动作指导、饮食营养管理和数据记录分析服务。

### 支持平台

| 平台 | 状态 |
|------|------|
| 飞书聊天机器人 | MVP |
| Web SPA | MVP 后迭代 |
| macOS Desktop | 未来规划 |

---

## 2. 系统架构概览

```
┌─────────────────────────────────────────────────────────┐
│                      客户端层                            │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
│  │ Web SPA  │  │ 飞书机器人 │  │macOS 桌面端│ (Future)   │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘             │
└───────┼──────────────┼──────────────┼───────────────────┘
        │              │              │
        ▼              ▼              ▼
┌─────────────────────────────────────────────────────────┐
│                   API Gateway (Axum)                     │
│         认证 / 限流 / 路由 / CORS / 日志                 │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│  用户服务     │ │  健身服务     │ │  AI 服务      │
│  认证/鉴权    │ │  计划/动作    │ │  LLM 调用     │
│  个人档案     │ │  饮食/记录    │ │  Prompt 管理  │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       ▼                ▼                ▼
┌─────────────────────────────────────────────────────────┐
│                    数据层                                │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
│  │PostgreSQL│  │  Redis   ��� │   S3/OSS  │             │
│  │ 主数据库  │  │ 缓存/会话 │  │ 文件存储   │             │
│  └──────────┘  └──────────┘  └───────────┘             │
└─────────────────────────────────────────────────────────┘
```

---

## 3. 技术选型

| 组件 | 选型 | Crate |
|------|------|-------|
| 语言 | Rust (Edition 2024) | - |
| Web 框架 | Axum | `axum` |
| ORM | SeaORM 2.0 | `sea-orm` |
| 数据库 | PostgreSQL (开发用 SQLite) | - |
| 缓存 | Redis | `redis` |
| LLM 集成 | async-openai | `async-openai` |
| 飞书 SDK | openlark | `openlark` |
| 认证 | JWT + Argon2 | `jsonwebtoken`, `argon2` |
| HTTP 客户端 | reqwest | `reqwest` |
| 序列化 | serde | `serde`, `serde_json` |
| 可观测性 | tracing | `tracing`, `tracing-subscriber` |
| 配置管理 | config | `config` |
| 参数校验 | validator | `validator` |
| HTTP 中间件 | tower-http | `tower-http` |
| 错误处理 | thiserror + anyhow | `thiserror`, `anyhow` |
| 部署 | Docker + 云服务器 | - |

---

## 4. 项目结构 (Rust Workspace)

```
fitness_service/
├── Cargo.toml                  # Workspace 根配置
├── config/
│   ├── default.toml            # 默认配置
│   ├── development.toml        # 开发环境
│   └── production.toml         # 生产环境
│
├── crates/
│   ├── fitness-core/           # 核心共享层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs        # 统一错误定义
│   │   │   ├── config.rs       # 配置结构体
│   │   │   └── types.rs        # 共享类型 (UserId, ExerciseId 等)
│   │   └── Cargo.toml
│   │
│   ├── fitness-entity/         # 数据模型层 (SeaORM Entity)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── user.rs
│   │   │   ├── workout_plan.rs
│   │   │   ├── exercise.rs
│   │   │   ├── workout_log.rs
│   │   │   ├── nutrition_log.rs
│   │   │   ├── body_metrics.rs
│   │   │   ├── chat_session.rs
│   │   │   └── chat_message.rs
│   │   └── Cargo.toml
│   │
│   ├── fitness-migration/      # 数据库迁移
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── m20260101_000001_user.rs
│   │   │   ├── m20260101_000002_workout.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── fitness-service/        # 业务逻辑层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── user_service.rs
│   │   │   ├── plan_service.rs       # 训练计划
│   │   │   ├── exercise_service.rs   # 动作库
│   │   │   ├── nutrition_service.rs  # 饮食营养
│   │   │   ├── metrics_service.rs    # 数据记录
│   │   │   └── ai_service.rs         # AI 业务编排
│   │   └── Cargo.toml
│   │
│   ├── fitness-llm/            # LLM 集成层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs       # async-openai 封装
│   │   │   ├── prompt/         # Prompt 模板管理
│   │   │   │   ├── mod.rs
│   │   │   │   ├── plan_generation.rs
│   │   │   │   ├── nutrition_advice.rs
│   │   │   │   ├── form_check.rs
│   │   │   │   └── chat.rs
│   │   │   └── schema.rs       # LLM 输出的结构化解析
│   │   └── Cargo.toml
│   │
│   ├── fitness-handler/        # HTTP API 层 (Axum Handlers)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── middleware/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── auth.rs     # JWT 认证中间件
│   │   │   │   └── rate_limit.rs
│   │   │   ├── auth_handler.rs
│   │   │   ├── user_handler.rs
│   │   │   ├── plan_handler.rs
│   │   │   ├── exercise_handler.rs
│   │   │   ├── nutrition_handler.rs
│   │   │   ├── metrics_handler.rs
│   │   │   └── ai_chat_handler.rs    # AI 对话接口
│   │   └── Cargo.toml
│   │
│   ├── fitness-bot/            # 飞书机器人
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── handler.rs      # 消息事件处理
│   │   │   ├── command.rs      # 指令解析 (/plan, /log, /diet 等)
│   │   │   └── card.rs         # 飞书卡片消息构建
│   │   └── Cargo.toml
│   │
│   └── fitness-app/            # 主程序入口
│       ├── src/
│       │   ├── main.rs         # 组装所有模块, 启动服务
│       │   └── state.rs        # 应用共享状态
│       └── Cargo.toml
│
├── tests/                      # 集成测试
├── docs/                       # 文档
├── design/                     # 设计文档
├── Dockerfile
├── docker-compose.yml
└── .env.example
```

### 依赖关系

```
fitness-app
  ├── fitness-handler ──→ fitness-service ──→ fitness-entity ──→ fitness-core
  │                     │                    └── fitness-migration
  │                     └── fitness-llm
  └── fitness-bot ──→ fitness-service

所有 crate 都依赖 fitness-core (共享类型/错误/配置)
```

---

## 5. 核心数据模型

```sql
-- 用户
users (
    id UUID PRIMARY KEY,
    phone VARCHAR(20),
    email VARCHAR(255),
    password_hash VARCHAR(255),
    nickname VARCHAR(100),
    avatar VARCHAR(500),
    fitness_level VARCHAR(20),       -- beginner/intermediate/advanced
    gender VARCHAR(10),
    birth_date DATE,
    height DECIMAL(5,2),
    weight DECIMAL(5,2),
    feishu_open_id VARCHAR(100),
    created_at TIMESTAMP,
    updated_at TIMESTAMP
)

-- 训练计划
workout_plans (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    name VARCHAR(200),
    goal VARCHAR(100),               -- lose_weight/build_muscle/maintain/etc
    difficulty VARCHAR(20),
    duration_weeks INT,
    schedule_json JSONB,             -- {day: [exercise_id, ...]}
    status VARCHAR(20),              -- active/completed/paused
    ai_generated BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
)

-- 动作库
exercises (
    id UUID PRIMARY KEY,
    name VARCHAR(200),
    category VARCHAR(50),            -- strength/cardio/flexibility/etc
    muscle_groups JSONB,             -- ["chest", "triceps"]
    equipment VARCHAR(200),
    difficulty VARCHAR(20),
    description TEXT,
    video_url VARCHAR(500),
    standard_json JSONB,             -- 标准姿态参数
    created_at TIMESTAMP
)

-- 训练记录
workout_logs (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    plan_id UUID REFERENCES workout_plans(id),
    exercise_id UUID REFERENCES exercises(id),
    sets_json JSONB,                 -- [{reps, weight, duration, rest}]
    duration_minutes INT,
    calories INT,
    notes TEXT,
    completed_at TIMESTAMP
)

-- 饮食记录
nutrition_logs (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    meal_type VARCHAR(20),           -- breakfast/lunch/dinner/snack
    foods_json JSONB,                -- [{name, amount, unit, calories, ...}]
    calories DECIMAL(7,2),
    protein DECIMAL(7,2),
    carbs DECIMAL(7,2),
    fat DECIMAL(7,2),
    ai_suggested BOOLEAN DEFAULT FALSE,
    logged_at TIMESTAMP
)

-- 身体指标
body_metrics (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    weight DECIMAL(5,2),
    body_fat DECIMAL(3,1),
    muscle_mass DECIMAL(4,1),
    waist DECIMAL(4,1),
    chest DECIMAL(4,1),
    arm DECIMAL(4,1),
    thigh DECIMAL(4,1),
    recorded_at TIMESTAMP
)

-- AI 对话
chat_sessions (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    platform VARCHAR(20),            -- web/feishu
    title VARCHAR(200),
    created_at TIMESTAMP
)

chat_messages (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES chat_sessions(id),
    role VARCHAR(20),                -- user/assistant/system
    content TEXT,
    metadata_json JSONB,
    created_at TIMESTAMP
)
```

---

## 6. API 设计

### 6.1 认证相关

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/auth/register` | 注册 |
| POST | `/api/v1/auth/login` | 登录 |
| POST | `/api/v1/auth/refresh` | 刷新 Token |
| POST | `/api/v1/auth/feishu` | 飞书 OAuth 登录 |

### 6.2 用户档案

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/users/me` | 获取个人信息 |
| PUT | `/api/v1/users/me` | 更新个人信息 |
| PUT | `/api/v1/users/fitness-profile` | 更新健身档案 (水平/目标/体测) |

### 6.3 训练计划 (AI 驱动)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/plans/generate` | AI 生成训练计划 |
| GET | `/api/v1/plans` | 列出我的计划 |
| GET | `/api/v1/plans/:id` | 计划详情 |
| PUT | `/api/v1/plans/:id` | 修改计划 |
| POST | `/api/v1/plans/:id/adjust` | AI 调整计划 |

### 6.4 动作库

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/exercises` | 搜索/筛选动作 |
| GET | `/api/v1/exercises/:id` | 动作详情 |

### 6.5 训练记录

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/workout-logs` | 记录训练 |
| GET | `/api/v1/workout-logs` | 查询记录 |
| GET | `/api/v1/workout-logs/stats` | 训练统计 |

### 6.6 饮食营养 (AI 驱动)

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/nutrition/analyze` | AI 分析饮食 |
| POST | `/api/v1/nutrition/suggest` | AI 推荐食谱 |
| GET | `/api/v1/nutrition/logs` | 饮食记录 |
| POST | `/api/v1/nutrition/logs` | 创建饮食记录 |

### 6.7 身体数据

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/metrics` | 获取指标历史 |
| POST | `/api/v1/metrics` | 记录指标 |
| GET | `/api/v1/metrics/trends` | 趋势分析 |

### 6.8 AI 对话

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/chat/sessions` | 创建对话 |
| POST | `/api/v1/chat/sessions/:id/messages` | 发送消息 (SSE 流式) |
| GET | `/api/v1/chat/sessions` | 对话列表 |
| GET | `/api/v1/chat/sessions/:id/messages` | 获取历史消息 |

### 6.9 飞书回调

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/feishu/event` | 飞书事件回调 |
| POST | `/api/v1/feishu/challenge` | 飞书 URL 验证 |

---

## 7. AI 集成架构

```
┌────────────────────────────────────────────┐
│              fitness-llm crate              │
│                                            │
│  ┌─────────────┐    ┌──────────────────┐  │
│  │ Prompt Engine │    │  Schema Parser   │  │
│  │              │    │                  │  │
│  │ ┌─────────┐ │    │  LLM 输出 → Rust  │  │
│  │ │ 训练计划 │ │    │  结构体 (serde)   │  │
│  │ │ 饮食建议 │ │    │                  │  │
│  │ │ 动作纠正 │ │    └──────────────────┘  │
│  │ │ 自由对话 │ │                          │
│  │ └─────────┘ │    ┌──────────────────┐  │
│  └─────────────┘    │  Provider Router  │  │
│                     │                  │  │
│                     │  OpenAI ──┐      │  │
│                     │  Claude ──┤      │  │
│                     │  本地 ────┘      │  │
│                     └──────────────────┘  │
└────────────────────────────────────────────┘
```

### 核心策略

- **结构化输出 (Structured Output)**: 通过 JSON Schema 约束 LLM 输出格式，确保训练计划、营养建议等可被程序解析
- **Prompt 模板化**: 每个场景独立 Prompt 模板，包含系统角色 + 用户上下文注入
- **多 Provider 路由**: 统一接口，支持 OpenAI / Claude / 本地模型切换
- **流式响应**: 对话场景使用 SSE 流式返回，计划生成可后台异步

### LLM 场景映射

| 场景 | Prompt 模板 | 输出格式 | 响应方式 |
|------|------------|---------|---------|
| 训练计划生成 | `plan_generation` | JSON (WorkoutPlan) | 异步 + 轮询 |
| 饮食建议 | `nutrition_advice` | JSON (MealPlan) | 同步/SSE |
| 动作纠正 (文字) | `form_check` | 自然语言 | SSE 流式 |
| 自由对话 | `chat` | 自然语言 | SSE 流式 |

---

## 8. 飞书机器人交互设计

### 指令系统

| 指令 | 功能 | 返回形式 |
|------|------|---------|
| `/plan <目标>` | AI 生成训练计划 | 飞书卡片 |
| `/log <内容>` | 记录训练/饮食 (智能识别) | 确认卡片 |
| `/diet <需求>` | 饮食建议 | 营养分析卡片 |
| `/stats` | 查看数据统计 | 图表卡片 |
| `/help` | 帮助信息 | 帮助卡片 |

### 自然语言交互

直接输入问题 → AI 对话模式，上下文关联用户健身数据

```
用户: "今天该练什么？"
Bot:  根据你的训练计划，今天练「胸部 + 三头肌」...
      [显示训练详情卡片]

用户: "我太累了能调整吗？"
Bot:  好的，我帮你把今天调整为轻松模式...
      [显示调整后卡片]
```

### 事件处理流程

```
飞书 Server → POST /api/v1/feishu/event
  → 签名验证
  → 事件类型路由
    ├── message.text → Command 解析 → 业务处理 → 返回卡片
    ├── message.image → 暂不支持 (MV 后迭代视觉)
    └── url_verification → challenge 响应
```

---

## 9. 认证体系

### 多平台认证架构

```
                    +-----------+
                    | Auth 服务  |
                    | (Axum API) |
                    +-----+-----+
                          |
           +--------------+--------------+
           |              |              |
       Web (SPA)    Feishu Bot    Desktop (Tauri)
      JWT+OAuth2    Feishu OAuth2  JWT+OAuth2 PKCE
```

### 认证策略

| 平台 | 认证方式 | 说明 |
|------|---------|------|
| Web | JWT (Email/Phone + 密码) | 标准登录，支持 OAuth 扩展 |
| 飞书 | Feishu OAuth2 | 通过飞书授权获取 open_id，映射到内部用户 |
| Desktop | OAuth2 PKCE | 无 client secret 的安全方案 |

### Token 策略

- Access Token: 15 分钟过期
- Refresh Token: 7 天过期
- Refresh 时自动轮换 Token

---

## 10. MVP 范围与优先级

| 优先级 | 功能模块 | 说明 |
|--------|---------|------|
| **P0** | 用户注册/登录 | 基础认证体系 |
| **P0** | 健身档案管理 | 收集用户身体数据、目标、水平 |
| **P0** | AI 训练计划生成 | 核心卖点，LLM 驱动 |
| **P0** | AI 自由对话 | 健身知识问答、通用交互入口 |
| **P0** | 飞书机器人 | MVP 主要触达渠道 |
| **P1** | 训练记录 | 用户记录完成情况 |
| **P1** | 饮食记录 + AI 建议 | 基础饮食管理 |
| **P1** | 身体数据记录 | 体重等指标追踪 |
| **P2** | 动作库 + 文字指导 | MVP 阶段不含视觉识别 |
| **P2** | 数据统计/趋势 | 可视化分析数据 |
| **P3** | 动作视觉纠正 | 需要 Pose Estimation，MVP 后迭代 |
| **P3** | Web 前端 | MVP 后迭代 |
| **Future** | macOS Desktop | Tauri + API 复用 |

---

## 11. 开发路线图

### Phase 1: 基础框架搭建 (Week 1-2)

- Workspace 初始化 + 各 crate 骨架
- PostgreSQL + SeaORM + 数据库迁移
- 用户认证体系 (JWT + Argon2)
- 基础 CRUD API (用户、健身档案)

### Phase 2: AI 核心能力 (Week 3-4)

- `fitness-llm` crate (async-openai 集成)
- Prompt 模板系统 (训练计划、对话)
- AI 对话 API (SSE 流式)
- 训练计划生成 API (结构化输出)

### Phase 3: 飞书机器人 (Week 5-6)

- `fitness-bot` crate (openlark 集成)
- 指令解析 + 卡片消息
- 飞书 OAuth 登录
- 端到端联调

### Phase 4: 完善功能 (Week 7-8)

- 训练记录 + AI 分析
- 饮食记录 + AI 建议
- 身体数据记录 + 趋势
- 数据统计 API
- 集成测试 + Docker 部署

---

## 12. 部署架构

```
┌───────────────────────────────────────────┐
│                 云服务器                    │
│  ┌──────────┐  ┌──────────┐               │
│  │  Nginx   │  │ Docker   │               │
│  │ (反向代理)│  │ Compose  │               │
│  └────┬─────┘  └────┬─────┘               │
│       │             │                      │
│       ▼             ▼                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │ fitness  │  │PostgreSQL│  │  Redis   ││
│  │  app     │  │  :5432   │  │  :6379   ││
│  │  :8080   │  └──────────┘  └──────────┘│
│  └──────────┘                             │
└───────────────────────────────────────────┘
```

### Docker Compose 服务列表

| 服务 | 端口 | 说明 |
|------|------|------|
| `fitness-app` | 8080 | Rust 主服务 |
| `postgres` | 5432 | 主数据库 |
| `redis` | 6379 | 缓存/会话 |

---

## 13. 后续升级方向

| 阶段 | 内容 |
|------|------|
| v1.1 | Web 前端 (React/Next.js)，复用现有 API |
| v1.2 | 动作视觉识别 (Pose Estimation)，客户端 MediaPipe + 服务端 LLM 分析 |
| v1.3 | macOS Desktop (Tauri)，本地 SQLite + 远端 API 同步 |
| v1.4 | 社区功能 (训练计划分享、排行榜) |
| v1.5 | 可穿戴设备数据接入 (Apple Health, 运动手表) |
