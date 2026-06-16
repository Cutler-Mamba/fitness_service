use super::PromptTemplate;

pub fn form_check() -> PromptTemplate {
    PromptTemplate::new(
        r#"你是一位专业的健身动作指导教练。根据用户描述的动作执行情况，给出专业的纠正建议和注意事项。

分析用户描述的动作，指出可能的问题，并给出正确的执行方法和安全建议。"#,
    )
}
