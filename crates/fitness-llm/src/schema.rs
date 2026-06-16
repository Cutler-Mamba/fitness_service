use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPlanOutput {
    pub name: String,
    pub goal: String,
    pub difficulty: String,
    pub duration_weeks: i32,
    pub schedule: std::collections::HashMap<String, Vec<String>>,
    pub exercises: Vec<PlanExerciseOutput>,
    pub tips: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExerciseOutput {
    pub name: String,
    pub sets: i32,
    pub reps: String,
    pub rest_seconds: i32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionAnalysisOutput {
    pub total_calories: f64,
    pub protein_g: f64,
    pub carbs_g: f64,
    pub fat_g: f64,
    pub assessment: String,
    pub suggestions: Vec<String>,
}
