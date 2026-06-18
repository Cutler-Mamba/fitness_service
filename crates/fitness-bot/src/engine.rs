use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fitness_core::{
    error::AppError,
    types::{FitnessGoal, FitnessProfile},
};
use fitness_service::{AiService, TenantService};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::command::{self, BotCommand};
use crate::ilink::IlinkSession;
use crate::platform::{IncomingMessage, MessagingPlatform};

const HELP_TEXT: &str = "\
**AI 健身助手 - 帮助**

支持以下指令：
- `/plan [目标]` - 生成个性化训练计划
- `/log <内容>` - 记录训练数据
- `/diet [要求]` - 获取饮食营养建议
- `/stats` - 查看健身统计
- `/help` - 显示此帮助

或者直接输入任何健身相关问题，我会为你解答！";

const DEDUP_WINDOW_SECS: u64 = 300;

pub struct BotEngine {
    ai_service: Arc<AiService>,
    tenant_service: Arc<TenantService>,
    session: Arc<IlinkSession>,
    recent_messages: RwLock<HashMap<String, Instant>>,
}

impl BotEngine {
    pub fn new(
        ai_service: Arc<AiService>,
        tenant_service: Arc<TenantService>,
        session: Arc<IlinkSession>,
    ) -> Self {
        Self {
            ai_service,
            tenant_service,
            session,
            recent_messages: RwLock::new(HashMap::new()),
        }
    }

    pub async fn handle_message(
        &self,
        platform: &dyn MessagingPlatform,
        msg: IncomingMessage,
    ) {
        let dedup_key = platform.dedup_key(&msg);

        if self.is_duplicate(&dedup_key).await {
            debug!("Duplicate message ignored: {}", dedup_key);
            return;
        }

        info!(
            "Handling message from {} on {}: {}",
            msg.user_id,
            platform.name(),
            &msg.text[..msg.text.len().min(100)]
        );

        match self.process_message(platform, &msg).await {
            Ok(()) => {}
            Err(e) => {
                error!("Error handling message from {}: {}", msg.user_id, e);
                let _ = platform
                    .send_text(&msg.user_id, "抱歉，处理你的消息时出错了，请稍后再试。", None)
                    .await;
            }
        }
    }

    async fn process_message(
        &self,
        platform: &dyn MessagingPlatform,
        msg: &IncomingMessage,
    ) -> Result<(), AppError> {
        let tenant = self
            .tenant_service
            .find_or_create(&msg.user_id, None)
            .await?;

        if tenant.status != "active" {
            platform
                .send_text(
                    &msg.user_id,
                    "您好！服务暂未开通，请联系管理员激活账户。",
                    msg.context_token.as_deref(),
                )
                .await?;
            return Ok(());
        }

        let cmd = command::parse_command(&msg.text);

        let reply = match cmd {
            BotCommand::Plan { goal } => {
                let profile = build_profile_from_goal(goal.as_deref());
                match self.ai_service.generate_plan(&profile).await {
                    Ok(plan) => format_plan_response(&plan),
                    Err(e) => {
                        error!("Plan generation failed: {}", e);
                        "抱歉，生成训练计划时出错了，请稍后再试。".to_string()
                    }
                }
            }
            BotCommand::Diet { request } => {
                let food_input = request.unwrap_or_else(|| msg.text.clone());
                let profile = Some(FitnessProfile {
                    level: None,
                    goal: Some(FitnessGoal::LoseWeight),
                    height_cm: None,
                    weight_kg: None,
                    gender: None,
                    birth_date: None,
                    weekly_days_available: None,
                    minutes_per_session: None,
                    equipment_available: None,
                    injuries_or_limitations: None,
                });
                match self
                    .ai_service
                    .analyze_nutrition(&food_input, profile.as_ref())
                    .await
                {
                    Ok(analysis) => format_nutrition_response(&analysis),
                    Err(e) => {
                        error!("Nutrition analysis failed: {}", e);
                        "抱歉，分析饮食时出错了，请稍后再试。".to_string()
                    }
                }
            }
            BotCommand::Help => HELP_TEXT.to_string(),
            BotCommand::Log { .. } | BotCommand::Stats => {
                "该功能即将上线，敬请期待！\n\n发送 /help 查看当前支持的指令。".to_string()
            }
            BotCommand::Unknown { text } => {
                match self
                    .ai_service
                    .chat(&text, None)
                    .await
                {
                    Ok(reply) => reply,
                    Err(e) => {
                        error!("AI chat failed: {}", e);
                        "抱歉，AI 服务暂时不可用，请稍后再试。".to_string()
                    }
                }
            }
        };

        let chunks = split_text(&reply, 4000);
        let total = chunks.len();

        for (i, chunk) in chunks.iter().enumerate() {
            let ctx = if i == 0 {
                msg.context_token.as_deref()
            } else {
                None
            };

            if let Err(e) = platform.send_text(&msg.user_id, chunk, ctx).await {
                error!("Failed to send message chunk {}/{}: {}", i + 1, total, e);
            }

            if total > 1 && i < total - 1 {
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        }

        if let Some(token) = &msg.context_token {
            self.session.set_context_token(&msg.user_id, token.clone());
        }

        Ok(())
    }

    async fn is_duplicate(&self, key: &str) -> bool {
        let mut map = self.recent_messages.write().await;

        map.retain(|_, ts| ts.elapsed() < Duration::from_secs(DEDUP_WINDOW_SECS));

        if map.contains_key(key) {
            return true;
        }

        map.insert(key.to_string(), Instant::now());
        false
    }
}

fn build_profile_from_goal(goal: Option<&str>) -> FitnessProfile {
    let fitness_goal = goal.and_then(|g| match g {
        "减脂" | "减肥" | "lose_weight" => Some(FitnessGoal::LoseWeight),
        "增肌" | "build_muscle" => Some(FitnessGoal::BuildMuscle),
        "维持" | "maintain" => Some(FitnessGoal::Maintain),
        "耐力" | "endurance" => Some(FitnessGoal::IncreaseEndurance),
        "柔韧" | "flexibility" => Some(FitnessGoal::ImproveFlexibility),
        _ => None,
    });

    FitnessProfile {
        level: None,
        goal: fitness_goal,
        height_cm: None,
        weight_kg: None,
        gender: None,
        birth_date: None,
        weekly_days_available: Some(3),
        minutes_per_session: Some(60),
        equipment_available: None,
        injuries_or_limitations: None,
    }
}

fn format_plan_response(plan: &fitness_llm::schema::GeneratedPlanOutput) -> String {
    let mut resp = format!(
        "**{}**\n目标: {} | 难度: {} | {}周\n\n",
        plan.name, plan.goal, plan.difficulty, plan.duration_weeks
    );

    resp.push_str("**训练安排：**\n");
    for (day, exercises) in &plan.schedule {
        resp.push_str(&format!("- {}: {}\n", day, exercises.join("、")));
    }

    if !plan.exercises.is_empty() {
        resp.push_str("\n**动作详情：**\n");
        for ex in &plan.exercises {
            resp.push_str(&format!(
                "- **{}**: {}组 x {}次, 休息{}秒\n",
                ex.name, ex.sets, ex.reps, ex.rest_seconds
            ));
            if !ex.notes.is_empty() {
                resp.push_str(&format!("  _{}\n", ex.notes));
            }
        }
    }

    if !plan.tips.is_empty() {
        resp.push_str("\n**训练建议：**\n");
        for tip in &plan.tips {
            resp.push_str(&format!("- {}\n", tip));
        }
    }

    resp
}

fn format_nutrition_response(analysis: &fitness_llm::schema::NutritionAnalysisOutput) -> String {
    let mut resp = format!(
        "**营养分析**\n\n总热量: {:.0} kcal | 蛋白质: {:.0}g | 碳水: {:.0}g | 脂肪: {:.0}g\n\n",
        analysis.total_calories,
        analysis.protein_g,
        analysis.carbs_g,
        analysis.fat_g
    );

    if !analysis.assessment.is_empty() {
        resp.push_str(&format!("**评估**: {}\n\n", analysis.assessment));
    }

    if !analysis.suggestions.is_empty() {
        resp.push_str("**建议**:\n");
        for s in &analysis.suggestions {
            resp.push_str(&format!("- {}\n", s));
        }
    }

    resp
}

pub fn split_text(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= max_len {
            chunks.push(remaining.to_string());
            break;
        }

        let split_at = find_split_point(remaining, max_len);
        chunks.push(remaining[..split_at].trim_end().to_string());
        remaining = remaining[split_at..].trim_start();
    }

    chunks
}

pub fn find_split_point(text: &str, max_len: usize) -> usize {
    let slice = &text[..max_len.min(text.len())];

    for pattern in &["\n\n", "\n", ". ", "。", "！", "？", " "] {
        if let Some(pos) = slice.rfind(pattern) {
            let candidate = pos + pattern.len();
            if candidate > max_len / 2 {
                return candidate;
            }
        }
    }

    max_len
}
