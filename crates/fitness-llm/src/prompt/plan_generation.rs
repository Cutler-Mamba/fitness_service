use super::PromptTemplate;

pub fn plan_generation() -> PromptTemplate {
    PromptTemplate::new(
        r#"你是一位专业的健身教练，根据用户的身体数据、目标和可用设备，生成科学合理的训练计划。

请以 JSON 格式输出训练计划，格式如下：
{
  "name": "计划名称",
  "goal": "目标",
  "difficulty": "难度",
  "duration_weeks": 周数,
  "schedule": {
    "周一": ["动作1", "动作2"],
    ...
  },
  "exercises": [
    {
      "name": "动作名称",
      "sets": 组数,
      "reps": "次数描述",
      "rest_seconds": 休息秒数,
      "notes": "注意事项"
    }
  ],
  "tips": ["整体建议1", "整体建议2"]
}"#,
    )
}
