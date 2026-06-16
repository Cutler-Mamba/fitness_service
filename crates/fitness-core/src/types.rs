use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;
pub type PlanId = Uuid;
pub type ExerciseId = Uuid;
pub type WorkoutLogId = Uuid;
pub type NutritionLogId = Uuid;
pub type SessionId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u64,
    pub page_size: u64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FitnessLevel {
    Beginner,
    Intermediate,
    Advanced,
}

impl std::fmt::Display for FitnessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FitnessLevel::Beginner => write!(f, "beginner"),
            FitnessLevel::Intermediate => write!(f, "intermediate"),
            FitnessLevel::Advanced => write!(f, "advanced"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FitnessGoal {
    LoseWeight,
    BuildMuscle,
    Maintain,
    IncreaseEndurance,
    ImproveFlexibility,
}

impl std::fmt::Display for FitnessGoal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FitnessGoal::LoseWeight => write!(f, "lose_weight"),
            FitnessGoal::BuildMuscle => write!(f, "build_muscle"),
            FitnessGoal::Maintain => write!(f, "maintain"),
            FitnessGoal::IncreaseEndurance => write!(f, "increase_endurance"),
            FitnessGoal::ImproveFlexibility => write!(f, "improve_flexibility"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    Active,
    Completed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExerciseCategory {
    Strength,
    Cardio,
    Flexibility,
    Balance,
    Plyometrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessProfile {
    pub level: Option<FitnessLevel>,
    pub goal: Option<FitnessGoal>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub weekly_days_available: Option<i32>,
    pub minutes_per_session: Option<i32>,
    pub equipment_available: Option<Vec<String>>,
    pub injuries_or_limitations: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutSet {
    pub reps: Option<i32>,
    pub weight_kg: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub rest_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodItem {
    pub name: String,
    pub amount: f64,
    pub unit: String,
    pub calories: Option<f64>,
    pub protein_g: Option<f64>,
    pub carbs_g: Option<f64>,
    pub fat_g: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
