use std::{str::FromStr, string::ParseError};

use serde::Serialize;
use tracing::debug;

#[derive(Debug, Serialize)]
pub enum NutritionType {
    AdditionalSugar,
    Ash,
    B1,
    B12,
    B2,
    B3,
    B5,
    B6,
    B8,
    Bicarbonate,
    Caffein,
    Calcium,
    Carb,
    Cellulose,
    Chloride,
    Chlorine,
    Cholesterol,
    Choline,
    Copper,
    Energy,
    Fat,
    Fiber,
    Fluorine,
    FolicAcid,
    Humidity,
    Iodine,
    Iron,
    Magnesium,
    Manganese,
    Nitrate,
    Nucleotide,
    Omega3,
    Omega6,
    Phosphorus,
    Polyol,
    Potassium,
    Protein,
    SaturatedFat,
    Selenium,
    Silica,
    Sodium,
    Sugar,
    Sulfur,
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
        if s.contains("מתוכם סוכר מוסף") {
            return Ok(NutritionType::AdditionalSugar);
        }
        if s.contains("אפר") {
            return Ok(NutritionType::Ash);
        }
        if s.contains("דו פחמות") {
            return Ok(NutritionType::Bicarbonate);
        }
        if s.contains("תאית") {
            return Ok(NutritionType::Cellulose);
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
        if s.contains("נתרן") || s.contains("מלח") {
            return Ok(NutritionType::Sodium);
        }
        if s.contains("שומן רווי") {
            return Ok(NutritionType::SaturatedFat);
        }
        if s.contains("חומצות שומן טרנס") || s.contains("חומצות שומן טראנס")
        {
            return Ok(NutritionType::TransFat);
        }
        if s.contains("יוד") {
            return Ok(NutritionType::Iodine);
        }
        if s.contains("B1 ויטמין") || s.contains("ויטמין B1") {
            return Ok(NutritionType::B1);
        }
        if s.contains("A ויטמין") || s.contains("ויטמין A") {
            return Ok(NutritionType::VitaminA);
        }
        if s.contains("B2 ויטמין") || s.contains("ויטמין B2") {
            return Ok(NutritionType::B2);
        }
        if s.contains("B3 ויטמין")
            || s.contains("ויטמין B3")
            || s.contains("ניאצין")
            || s.contains("ניקוטינאמיד")
        {
            return Ok(NutritionType::B3);
        }
        if s.contains("B5 ויטמין") || s.contains("ויטמין B5") {
            return Ok(NutritionType::B5);
        }
        if s.contains("B6 ויטמין") || s.contains("ויטמין B6") {
            return Ok(NutritionType::B6);
        }
        if s.contains("ביוטין") {
            return Ok(NutritionType::B8);
        }
        if s.contains("B12 ויטמין") || s.contains("ויטמין B12") {
            return Ok(NutritionType::B12);
        }
        if s.contains("C ויטמין") || s.contains("ויטמין C") || s.contains("חומצה אסקורבית")
        {
            return Ok(NutritionType::VitaminC);
        }
        if s.contains("D ויטמין") || s.contains("ויטמין D") {
            return Ok(NutritionType::VitaminD);
        }
        if s.contains("E ויטמין") || s.contains("ויטמין E") {
            return Ok(NutritionType::VitaminE);
        }
        if s.contains("K ויטמין") || s.contains("ויטמין K") {
            return Ok(NutritionType::VitaminK);
        }
        if s.contains("לחות") || s.contains("רטיבות") {
            return Ok(NutritionType::Humidity);
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
        if s.contains("חנקות") {
            return Ok(NutritionType::Nitrate);
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
        if s.contains("כלור") {
            return Ok(NutritionType::Chlorine);
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
        if s.contains("אומגה_3") || s.contains("אומגה 3") {
            return Ok(NutritionType::Omega3);
        }
        if s.contains("אומגה_6") {
            return Ok(NutritionType::Omega6);
        }
        if s.contains("גפרות") {
            return Ok(NutritionType::Sulfur);
        }
        if s.contains("סילקה") {
            return Ok(NutritionType::Silica);
        }
        if s.contains("פלואור") {
            return Ok(NutritionType::Fluorine);
        }
        debug!("Cannot find Nutrition Type for {s}");
        return Ok(NutritionType::Undefined(s.to_string()));
    }
}

#[derive(Debug, Serialize)]
pub enum Unit {
    #[serde(rename = "g")]
    Gram,
    #[serde(rename = "kcal")]
    Kcal,
    #[serde(rename = "µg")]
    Microgram,
    #[serde(rename = "mg")]
    Milligram,
    #[serde(rename = "%")]
    Percent,
    #[serde(rename = "teaspoon")]
    Teaspoon,
    Undefined(String),
}

impl Unit {
    pub fn str(&self) -> String {
        match self {
            Unit::Gram => "g",
            Unit::Kcal => "kcal",
            Unit::Microgram => "µg",
            Unit::Milligram => "mg",
            Unit::Percent => "%",
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
        if s == "גרם" || s == "גר" {
            return Ok(Unit::Gram);
        }
        if s == "קל" || s == "קלוריות" || s == "קק\"ל" {
            return Ok(Unit::Kcal);
        }
        if s == "מג" || s == "מגn" {
            return Ok(Unit::Milligram);
        }
        if s == "מקג" {
            return Ok(Unit::Microgram);
        }
        if s == "%" {
            return Ok(Unit::Percent);
        }
        debug!("Cannot find Unit for {s}");
        return Ok(Unit::Undefined(s.to_string()));
    }
}

#[derive(Debug, Serialize)]
pub struct NutritionalValue {
    pub number: String,
    pub unit: Unit,
    pub nutrition_type: NutritionType,
    #[serde(skip_serializing_if = "is_false")]
    pub less_than: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

impl NutritionalValue {
    pub fn new(number: String, unit: String, nutrition_type: String) -> Option<NutritionalValue> {
        let unit = if unit == "קלוריותn" {
            "קלוריות"
        } else if unit == "מגn" {
            "מג"
        } else if unit == "גרםn" {
            "גרם"
        } else if unit.ends_with("nnn") {
            unit.strip_suffix("nnn").unwrap()
        } else {
            unit.as_str()
        };
        let nutrition_type = if nutrition_type.ends_with(&format!(" ({unit})")) {
            nutrition_type.strip_suffix(&format!(" ({unit})")).unwrap()
        } else {
            nutrition_type.as_str()
        };

        if unit == "" && nutrition_type == "כפיות סוכר" {
            return Some(NutritionalValue {
                number: number,
                unit: Unit::Teaspoon,
                nutrition_type: NutritionType::Sugar,
                less_than: false,
            });
        }
        if number.is_empty() || unit.is_empty() || nutrition_type.is_empty() {
            return None;
        }
        return Some(NutritionalValue {
            number: number,
            unit: unit.parse().unwrap(),
            nutrition_type: nutrition_type.parse().unwrap(),
            less_than: false,
        });
    }
    pub fn create(
        number: String,
        unit: String,
        nutrition_type: String,
        less_than: bool,
    ) -> Option<NutritionalValue> {
        NutritionalValue::new(number, unit, nutrition_type)
            .map(|n| NutritionalValue { less_than, ..n })
    }

    pub fn to_tuple(&self) -> (String, String, String) {
        (
            self.number.clone(),
            self.unit.str(),
            self.nutrition_type.str(),
        )
    }
}

#[derive(Debug, Serialize)]
pub struct NutritionalValues {
    pub size: Option<String>,
    pub values: Vec<NutritionalValue>,
}
