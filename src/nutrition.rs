use std::{str::FromStr, string::ParseError};

use tracing::debug;

#[derive(Debug)]
pub enum NutritionType {
    B1,
    B12,
    B2,
    B3,
    B5,
    B6,
    B8,
    Caffein,
    Calcium,
    Carb,
    Chloride,
    Cholesterol,
    Choline,
    Copper,
    Energy,
    Fat,
    Fiber,
    FolicAcid,
    Iron,
    Magnesium,
    Manganese,
    Nucleotide,
    Omega3,
    Omega6,
    Phosphorus,
    Polyol,
    Potassium,
    Protein,
    SaturatedFat,
    Selenium,
    Sodium,
    Sugar,
    Taurine,
    TransFat,
    VitaminA,
    VitaminC,
    VitaminD,
    VitaminE,
    VitaminK,
    Zinc,
    Undefined(String),
}
impl NutritionType {
    fn str(&self) -> String {
        format!("{self}")
    }
}
impl std::fmt::Display for NutritionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let NutritionType::Undefined(s) = self {
            write!(f, "{s}")
        } else {
            write!(f, "{:?}", self)
        }
    }
}
impl FromStr for NutritionType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("סיבים תזונתיים") {
            return Ok(NutritionType::Fiber);
        }
        if s.contains("סוכרים מתוך פחמימות") || s.contains("סוכרים") {
            return Ok(NutritionType::Sugar);
        }
        if s.contains("אנרגיה") {
            return Ok(NutritionType::Energy);
        }
        if s.contains("חלבונים") {
            return Ok(NutritionType::Protein);
        }
        if s.contains("פחמימות") {
            return Ok(NutritionType::Carb);
        }
        if s.contains("שומנים") {
            return Ok(NutritionType::Fat);
        }
        if s.contains("כולסטרול") {
            return Ok(NutritionType::Cholesterol);
        }
        if s.contains("נתרן") {
            return Ok(NutritionType::Sodium);
        }
        if s.contains("שומן רווי") {
            return Ok(NutritionType::SaturatedFat);
        }
        if s.contains("חומצות שומן טרנס") {
            return Ok(NutritionType::TransFat);
        }
        if s.contains("B1 ויטמין") {
            return Ok(NutritionType::B1);
        }
        if s.contains("A ויטמין") {
            return Ok(NutritionType::VitaminA);
        }
        if s.contains("B2 ויטמין") {
            return Ok(NutritionType::B2);
        }
        if s.contains("B3 ויטמין") || s.contains("ניאצין") || s.contains("ניקוטינאמיד")
        {
            return Ok(NutritionType::B3);
        }
        if s.contains("B5 ויטמין") {
            return Ok(NutritionType::B5);
        }
        if s.contains("B6 ויטמין") {
            return Ok(NutritionType::B6);
        }
        if s.contains("ביוטין") {
            return Ok(NutritionType::B8);
        }
        if s.contains("B12 ויטמין") {
            return Ok(NutritionType::B12);
        }
        if s.contains("C ויטמין") || s.contains("חומצה אסקורבית") {
            return Ok(NutritionType::VitaminC);
        }
        if s.contains("D ויטמין") {
            return Ok(NutritionType::VitaminD);
        }
        if s.contains("E ויטמין") || s.contains("ויטמין E") {
            return Ok(NutritionType::VitaminE);
        }
        if s.contains("K ויטמין") || s.contains("ויטמין K") {
            return Ok(NutritionType::VitaminK);
        }
        if s.contains("אבץ") {
            return Ok(NutritionType::Zinc);
        }
        if s.contains("אשלגן") {
            return Ok(NutritionType::Potassium);
        }
        if s.contains("ברזל") {
            return Ok(NutritionType::Iron);
        }
        if s.contains("מגנזיום") {
            return Ok(NutritionType::Magnesium);
        }
        if s.contains("רב כהלים") {
            return Ok(NutritionType::Polyol);
        }
        if s.contains("זרחן") {
            return Ok(NutritionType::Phosphorus);
        }
        if s.contains("חומצה פולית") {
            return Ok(NutritionType::FolicAcid);
        }
        if s.contains("קפאין") {
            return Ok(NutritionType::Caffein);
        }
        if s.contains("טאורין") {
            return Ok(NutritionType::Taurine);
        }
        if s.contains("סידן") {
            return Ok(NutritionType::Calcium);
        }
        if s.contains("מנגן") {
            return Ok(NutritionType::Manganese);
        }
        if s.contains("כולין") {
            return Ok(NutritionType::Choline);
        }
        if s.contains("כלוריד") {
            return Ok(NutritionType::Chloride);
        }
        if s.contains("נחושת") {
            return Ok(NutritionType::Copper);
        }
        if s.contains("סלניום") {
            return Ok(NutritionType::Selenium);
        }
        if s.contains("נוקלאוטידים") {
            return Ok(NutritionType::Nucleotide);
        }
        if s.contains("אומגה_3") {
            return Ok(NutritionType::Omega3);
        }
        if s.contains("אומגה_6") {
            return Ok(NutritionType::Omega6);
        }
        debug!("Cannot find Nutrition Type for {s}");
        return Ok(NutritionType::Undefined(s.to_string()));
    }
}

#[derive(Debug)]
pub enum Unit {
    Kcal,
    Microgram,
    Milligram,
    Gram,
    Teaspoon,
    Undefined(String),
}

impl Unit {
    pub fn str(&self) -> String {
        match self {
            Unit::Kcal => "kcal",
            Unit::Microgram => "µg",
            Unit::Milligram => "mg",
            Unit::Gram => "g",
            Unit::Teaspoon => "teaspoon",
            Unit::Undefined(s) => s,
        }
        .to_string()
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}
impl FromStr for Unit {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "גרם" {
            return Ok(Unit::Gram);
        }
        if s == "קל" {
            return Ok(Unit::Kcal);
        }
        if s == "מג" {
            return Ok(Unit::Milligram);
        }
        if s == "מקג" {
            return Ok(Unit::Microgram);
        }
        debug!("Cannot find Unit for {s}");
        return Ok(Unit::Undefined(s.to_string()));
    }
}

#[derive(Debug)]
pub struct NutritionalValue {
    number: String,
    unit: Unit,
    nutrition_type: NutritionType,
}

impl NutritionalValue {
    pub fn new(number: String, unit: String, nutrition_type: String) -> Option<NutritionalValue> {
        if unit == "" && nutrition_type == "כפיות סוכר" {
            return Some(NutritionalValue {
                number: number,
                unit: Unit::Teaspoon,
                nutrition_type: NutritionType::Sugar,
            });
        }
        if number.is_empty() || unit.is_empty() || nutrition_type.is_empty() {
            return None;
        }
        return Some(NutritionalValue {
            number: number,
            unit: unit.parse().unwrap(),
            nutrition_type: nutrition_type.parse().unwrap(),
        });
    }
    pub fn to_tuple(&self) -> (String, String, String) {
        (
            self.number.clone(),
            self.unit.str(),
            self.nutrition_type.str(),
        )
    }
}
