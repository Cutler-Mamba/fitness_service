# Ubuntu 内网部署指南

> 适用于 Ubuntu 22.04/24.04，无需公网 IP，通过 iLink 长轮询接入微信。

---

## 1. 前置条件

- Ubuntu 22.04 或 24.04（x86_64）
- 可访问互联网（出站 HTTPS 到 `ilinkai.weixin.qq.com` 和 `api.openai.com`）
- 个人微信账号（用于扫码登录 iLink Bot）

---

## 2. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

---

## 3. 克隆项目

```bash
mkdir -p /opt
cd /opt
git clone https://github.com/Cutler-Mamba/fitness_service.git fitness
cd fitness
```

---

## 4. 构建

```bash
# Debug 构建（开发调试用）
cargo build

# 或 Release 构建（生产用）
cargo build --release
```

构建产物位置：
- Debug: `target/debug/fitness-app`, `fitness-cli`
- Release: `target/release/fitness-app`, `fitness-cli`

---

## 5. 配置环境变量

```bash
cp .env.example .env
# 编辑 .env 填写以下必填项：
```

**必填项**：

```ini
# LLM 配置（必填）
LLM_API_KEY=sk-your-api-key
LLM_MODEL=gpt-4o

# JWT 密钥（必填，随机生成）
JWT_SECRET=$(openssl rand -hex 32)

# 数据库（使用默认 SQLite 即可）
DATABASE_URL=sqlite:./data/fitness.db?mode=rwc
```

**微信 Bot 配置**（先配置通道，凭证通过 CLI 扫码获取）：

```ini
WECHAT_CHANNEL=ilink
WECHAT_CONTEXT_DIR=./data/weixin/context
```

---

## 6. 获取 iLink Bot 凭证（扫码登录）

```bash
# 运行扫码登录流程
cargo run -p fitness-cli -- wechat login
```

终端会显示二维码链接，用个人微信扫码并确认登录。成功后输出：

```
  WECHAT_ACCOUNT_ID=xxx
  WECHAT_TOKEN=xxx
```

将这两行添加到 `.env` 文件中。

---

## 7. 初始化数据库

```bash
# 执行数据库迁移
cargo run -p fitness-migrate -- up

# 或使用脚本
./script/migrate.sh up
```

---

## 8. 启动服务

### 方式一：直接运行（调试用）

```bash
RUST_LOG=fitness_app=debug cargo run
```

### 方式二：systemd 服务（生产用）

```bash
# 创建数据目录
mkdir -p /opt/fitness/data/weixin/context

# 创建运行用户
sudo useradd -r -s /bin/false -d /opt/fitness fitness

# 从 debug 或 release 目录拷贝二进制
sudo cp target/release/fitness-app /opt/fitness/
sudo cp target/release/fitness-cli /opt/fitness/
sudo cp target/release/fitness-migrate /opt/fitness/

# 拷贝 .env
sudo cp .env /opt/fitness/

# 设置权限
sudo chown -R fitness:fitness /opt/fitness

# 安装 systemd 服务
sudo cp deploy/fitness.service /etc/systemd/system/
sudo systemctl daemon-reload

# 编辑服务文件中的必要环境变量
sudo systemctl edit fitness-app

# 启动并设置开机自启
sudo systemctl enable fitness-app
sudo systemctl start fitness-app

# 查看状态
sudo systemctl status fitness-app

# 查看日志
sudo journalctl -u fitness-app -f
```

---

## 9. 管理租户

用户首次给 Bot 发消息后，会自动创建租户记录（状态为 `disabled`）。管理员需要手动激活：

```bash
# 列出所有租户
cargo run -p fitness-cli -- tenant list

# 激活租户
cargo run -p fitness-cli -- tenant status <WECHAT_USER_ID> active

# 查看租户详情
cargo run -p fitness-cli -- tenant show <WECHAT_USER_ID>
```

---

## 10. 验证服务

```bash
# 健康检查
curl http://127.0.0.1:8080/api/v1/auth/login -X POST \
  -H "Content-Type: application/json" \
  -d '{"login":"test","password":"test"}'

# 查看日志确认 iLink 轮询已启动
sudo journalctl -u fitness-app -f | grep iLink
```

预期看到：`iLink platform started, entering poll loop`

---

## 架构说明（内网模式）

```
 ┌────────────┐      长轮询 HTTPS       ┌──────────────────────┐
 │  Ubuntu 内网 │ ◄───────────────────► │  ilinkai.weixin.qq.com │
 │  fitness-app │    getupdates / send  │  (iLink Bot API)      │
 │  :8080       │                       └──────────┬───────────┘
 └──────────────┘                                  │
                                                   ▼
                                             ┌──────────┐
                                             │ 微信用户  │
                                             │ (个人微信) │
                                             └──────────┘
```

- 服务通过 HTTP 长轮询**主动拉取**消息（`GET /ilink/bot/getupdates`）
- 服务通过 HTTP POST **主动发送**消息（`POST /ilink/bot/sendmessage`）
- 所有通信均为**出站** HTTPS，无需公网 IP、无需开放入站端口
