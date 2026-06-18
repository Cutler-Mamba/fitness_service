use fitness_core::{error::AppError, types::FitnessProfile};
use fitness_llm::{LlmClient, schema::GeneratedPlanOutput, schema::NutritionAnalysisOutput};

pub struct AiService {
    llm_client: LlmClient,
}

impl AiService {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }

    pub async fn chat(&self, message: &str, profile: Option<&FitnessProfile>) -> Result<String, AppError> {
        let prompt = fitness_llm::prompt::chat::chat();
        let user_message = build_chat_message(message, profile);
        self.llm_client.chat(&prompt.system, &user_message).await
    }

    pub async fn generate_plan(&self, profile: &FitnessProfile) -> Result<GeneratedPlanOutput, AppError> {
        let prompt = fitness_llm::prompt::plan_generation::plan_generation();
        let user_message = build_profile_message(profile);

        let schema = serde_json::to_value(GeneratedPlanOutput {
            name: String::new(),
            goal: String::new(),
            difficulty: String::new(),
            duration_weeks: 0,
            schedule: Default::default(),
            exercises: vec![],
            tips: vec![],
        })
        .map_err(|e| AppError::Internal(format!("Failed to build schema: {}", e)))?;

        let raw = self
            .llm_client
            .chat_with_json_schema(&prompt.system, &user_message, "plan", schema)
            .await?;

        serde_json::from_str::<GeneratedPlanOutput>(&raw)
            .map_err(|e| AppError::LlmError(format!("Failed to parse plan: {}", e)))
    }

    pub async fn analyze_nutrition(&self, food_input: &str, profile: Option<&FitnessProfile>) -> Result<NutritionAnalysisOutput, AppError> {
        let prompt = fitness_llm::prompt::nutrition_advice::nutrition_advice();
        let mut user_message = format!("用户饮食内容：\n{}", food_input);
        if let Some(p) = profile {
            user_message.push_str(&format!("\n\n用户身体数据：{}", format_profile(p)));
        }

        let schema = serde_json::to_value(NutritionAnalysisOutput {
            total_calories: 0.0,
            protein_g: 0.0,
            carbs_g: 0.0,
            fat_g: 0.0,
            assessment: String::new(),
            suggestions: vec![],
        })
        .map_err(|e| AppError::Internal(format!("Failed to build schema: {}", e)))?;

        let raw = self
            .llm_client
            .chat_with_json_schema(&prompt.system, &user_message, "nutrition", schema)
            .await?;

        serde_json::from_str::<NutritionAnalysisOutput>(&raw)
            .map_err(|e| AppError::LlmError(format!("Failed to parse nutrition analysis: {}", e)))
    }
}

fn build_chat_message(message: &str, profile: Option<&FitnessProfile>) -> String {
    let mut msg = message.to_string();
    if let Some(p) = profile {
        msg.push_str(&format!("\n\n我的身体数据：{}", format_profile(p)));
    }
    msg
}

fn build_profile_message(profile: &FitnessProfile) -> String {
    format!(
        "请根据以下用户数据生成一份个性化训练计划：\n{}",
        format_profile(profile)
    )
}

fn format_profile(p: &FitnessProfile) -> String {
    let mut parts = Vec::new();

    if let Some(ref level) = p.level {
        parts.push(format!("健身水平: {}", level));
    }
    if let Some(ref goal) = p.goal {
        parts.push(format!("目标: {}", goal));
    }
    if let Some(h) = p.height_cm {
        parts.push(format!("身高: {}cm", h));
    }
    if let Some(w) = p.weight_kg {
        parts.push(format!("体重: {}kg", w));
    }
    if let Some(ref g) = p.gender {
        parts.push(format!("性别: {}", g));
    }
    if let Some(ref b) = p.birth_date {
        parts.push(format!("出生日期: {}", b));
    }
    if let Some(d) = p.weekly_days_available {
        parts.push(format!("每周可训练天数: {}", d));
    }
    if let Some(m) = p.minutes_per_session {
        parts.push(format!("每次训练时长: {}分钟", m));
    }
    if let Some(ref eq) = p.equipment_available {
        if !eq.is_empty() {
            parts.push(format!("可用设备: {}", eq.join("、")));
        }
    }
    if let Some(ref inj) = p.injuries_or_limitations {
        parts.push(format!("伤病/限制: {}", inj));
    }

    parts.join("; ")
}
