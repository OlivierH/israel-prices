use std::collections::HashMap;

use anyhow::Result;
use israel_prices::models::{ItemInfo, ItemKey};
use itertools::Itertools;
use multimap::MultiMap;
use tracing::info;
use tracing_subscriber::prelude::*;

enum Tag {
    AirPurifier,
    Alcohol,
    BabyAndNursingAccessories,
    Beer,
    BirthdayAccessories,
    Biscuits,
    BonbonsAndPralines,
    Bottles,
    Bread,
    BreakfastCereals,
    CandlesAndMatches,
    Candy,
    CannedFish,
    CannedTomatoes,
    CannedVegetables,
    CarbonatedDrinks,
    Cheese,
    CheeseAlternatives,
    Chocolate,
    ChocolateSnacks,
    ChocolateTablets,
    Cigarettes,
    CleaningAccessories,
    Concentrates,
    Cookies,
    CookiesAndBiscuits,
    CookiesPackaged,
    CookingSauce,
    Crackers,
    CreamyCheeses,
    DairyDrinks,
    Deodorant,
    DisposableDishes,
    FamilyIceCreams,
    Flour,
    FrozenFish,
    FrozenVegetables,
    Fruit,
    Gum,
    Jam,
    KitchenAccessories,
    Laundry,
    LaundrySoftener,
    Milk,
    MilkAndChocolateDrinks,
    NaturalCosmetics,
    Oils,
    Olives,
    PackagedCakes,
    PackagedSnacks,
    Pasta,
    PetFood,
    PlainYogurt,
    RiceCouscousAndPasta,
    RollsAndPitas,
    SaltedCheeses,
    SaltySnacks,
    ShampooAndConditioner,
    ShavingAndWaxingProducts,
    Soap,
    SoftDrinks,
    SoftDrinksFlavoredWater,
    Spices,
    TeaAndInfusions,
    Vegetables,
    Whisky,
    Wine,
    Yogurt,
    HouseCleaning,
    HairProductsAndAccessories,
    LeavesAndHerbs,
    Maadanim,
    ToiletAndBathCleaningProducts,
    JamsAndDips,
    PreparedSalads,
    Vitamins,
    Waffles,
    HealthSnacks,
    EnergyAndCerealSnacks,
    NaturalToiletries,
    FaceAndBodyCare,
    CoffeeAndCapsules,
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("annotate_tags=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting");

    let item_infos: HashMap<ItemKey, ItemInfo> = {
        let item_infos_file = std::io::BufReader::new(std::fs::File::open("item_infos.json")?);
        info!("Reading item_infos from item_infos.json");
        let item_infos: israel_prices::models::ItemInfos =
            serde_json::from_reader(item_infos_file)?;
        info!(
            "Read {} item_infos from item_infos.json",
            item_infos.data.len()
        );
        item_infos.data
    };

    #[derive(serde::Deserialize)]
    struct ScrapedDataWithCode {
        #[serde(rename = "Source")]
        source: String,
        #[serde(rename = "ItemCode")]
        itemcode: String,
        #[serde(rename = "Categories")]
        categories: Option<String>,
        #[serde(rename = "NutritionInfo")]
        nutrition_info: Option<String>,
        #[serde(rename = "Ingredients")]
        ingredients: Option<String>,
        #[serde(rename = "ImageUrls")]
        image_urls: Option<String>,
    }
    struct ScrapedData {
        source: String,
        categories: Vec<String>,
        nutrition_info: Option<String>,
        ingredients: Option<String>,
        image_urls: Option<String>,
    }

    // sqlite3 data.sqlite '.mode json' 'select * from ScrapedData;' > scraped_data.json
    let scraped_data = {
        let scraped_datafile = std::io::BufReader::new(std::fs::File::open("scraped_data.json")?);
        info!("Reading scraped_data from scraped_data.json");
        let scraped_data: Vec<ScrapedDataWithCode> = serde_json::from_reader(scraped_datafile)?;
        info!(
            "Read {} scraped_data from scraped_data.json",
            scraped_data.len()
        );
        scraped_data
            .into_iter()
            .map(|sd| {
                (
                    sd.itemcode,
                    ScrapedData {
                        source: sd.source,
                        categories: sd
                            .categories
                            .map(|s| serde_json::from_str::<Vec<String>>(&s).unwrap())
                            .unwrap_or_default(),
                        nutrition_info: sd.nutrition_info,
                        ingredients: sd.ingredients,
                        image_urls: sd.image_urls,
                    },
                )
            })
            .collect::<MultiMap<String, ScrapedData>>()
    };

    info!(
        "Got {} data from scraped_data.json",
        scraped_data.iter_all().map(|(x, y)| y.len()).sum::<usize>()
    );

    let mut numitemswithtags = 0;
    let numitems = item_infos.len();

    for (itemkey, iteminfo) in item_infos {
        let separator = ", ".to_owned();
        let iteminfo: israel_prices::models::ItemInfo = iteminfo;
        let itemcode = itemkey.item_code;
        let category_vecs = {
            if let Some(data) = scraped_data.get_vec(&itemcode.to_string()) {
                data
            } else {
                continue;
            }
        }
        .iter()
        .map(|d| (&d.categories, &d.source))
        .filter(|v: &(&Vec<String>, &String)| !v.0.is_empty())
        .unique()
        .collect::<Vec<(&Vec<String>, &String)>>();
        let mut hastag = false;
        for category_vec in &category_vecs {
            if let Some(category) = category_vec.0.last() {
                let tag = get_tag_for_string(&category);
                if tag.is_some() {
                    hastag = true;
                }
            }
        }
        if hastag {
            numitemswithtags += 1;
        } else {
            for category_vec in &category_vecs {
                if let Some(category) = category_vec.0.last() {
                    println!("!{}-{category}!", category_vec.1);
                }
            }
            if !category_vecs.is_empty() {
                // println!("{:?}", categoryvecs);
            }
        }
        // if !scraped_datas.isempty() {
        //     println!("{:?}", scraped_datas);
        // }
        // // .collect::<String>();
    }
    println!("Got {numitemswithtags} items with tags out of {numitems}");

    Ok(())
}

fn get_tag_for_string(s: &str) -> Option<Tag> {
    Some(match s {
        "אביזרי יום הולדת" => Tag::BirthdayAccessories,
        "אביזרי מטבח" => Tag::KitchenAccessories,
        "אביזרי ניקיון" => Tag::CleaningAccessories,
        "אביזרי תינוקות והנקה" => Tag::BabyAndNursingAccessories,
        "אבקות ונוזלי כביסה" => Tag::Laundry,
        "אורז, קוסקוס ופתיתים" => Tag::RiceCouscousAndPasta,
        "אלכוהול" => Tag::Alcohol,
        "אסלות ואמבטיות" => Tag::ToiletAndBathCleaningProducts,
        "בונבוניירה ופרלינים" => Tag::BonbonsAndPralines,
        "ביסקוויטים" => Tag::Biscuits,
        "בירה" => Tag::Beer,
        "בקבוקים" => Tag::Bottles,
        "גבינות מלוחות" => Tag::SaltedCheeses,
        "גבינות שמנת" => Tag::CreamyCheeses,
        "גבינות" => Tag::Cheese,
        "גלידות משפחתיות" => Tag::FamilyIceCreams,
        "דאודורנט" => Tag::Deodorant,
        "דאודורנטים" => Tag::Deodorant,
        "דגים קפואים" => Tag::FrozenFish,
        "דגני בוקר" => Tag::BreakfastCereals,
        "וויסקי" => Tag::Whisky,
        "וופלים" => Tag::Waffles,
        "ויטמינים" => Tag::Vitamins,
        "זיתים" => Tag::Olives,
        "חטיפי בריאות" => Tag::HealthSnacks,
        "חטיפי דגנים ואנרגיה" => Tag::EnergyAndCerealSnacks,
        "חטיפי שוקולד" => Tag::ChocolateSnacks,
        "חטיפים ארוזים" => Tag::PackagedSnacks,
        "חטיפים מלוחים" => Tag::SaltySnacks,
        "חלב" => Tag::Milk,
        "טבלאות שוקולד" => Tag::ChocolateTablets,
        "טואלטיקה טבעית" => Tag::NaturalToiletries,
        "טיפוח הגוף והפנים" => Tag::FaceAndBodyCare,
        "יוגורט לבן" => Tag::PlainYogurt,
        "יוגורט" => Tag::Yogurt,
        "יין" => Tag::Wine,
        "ירקות קפואים" => Tag::FrozenVegetables,
        "ירקות" => Tag::Vegetables,
        "כביסה" => Tag::Laundry,
        "כלים חד פעמיים" => Tag::DisposableDishes,
        "לחם" => Tag::Bread,
        "לחמניות ופיתות" => Tag::RollsAndPitas,
        "מוצרים ואביזרים לשיער" => Tag::HairProductsAndAccessories,
        "מזון לבעלי חיים" => Tag::PetFood,
        "מטהר אוויר" => Tag::AirPurifier,
        "ממרחים ומתבלים" => Tag::JamsAndDips,
        "מסטיקים" => Tag::Gum,
        "מעדנים" => Tag::Maadanim,
        "מרכך כביסה" => Tag::LaundrySoftener,
        "משקאות חלב ושוקו" => Tag::MilkAndChocolateDrinks,
        "משקאות חלב" => Tag::DairyDrinks,
        "משקאות מוגזים" => Tag::CarbonatedDrinks,
        "משקאות קלים, מים בטעמים" => Tag::SoftDrinksFlavoredWater,
        "משקאות קלים" => Tag::SoftDrinks,
        "ניקוי לבית" => Tag::HouseCleaning,
        "נרות וגפרורים" => Tag::CandlesAndMatches,
        "סבונים" => Tag::Soap,
        "סוכריות" => Tag::Candy,
        "סיגריות" => Tag::Cigarettes,
        "סלטים שונים" => Tag::PreparedSalads,
        "עוגות ארוזות" => Tag::PackagedCakes,
        "עוגיות ארוזות" => Tag::CookiesPackaged,
        "עוגיות וביסקוויטים" => Tag::CookiesAndBiscuits,
        "עוגיות" => Tag::Cookies,
        "עלים ועשבי תיבול" => Tag::LeavesAndHerbs,
        "פירות" => Tag::Fruit,
        "פסטה,אטריות ונודלס" => Tag::Pasta,
        "פסטות" => Tag::Pasta,
        "קוסמטיקה טבעית" => Tag::NaturalCosmetics,
        "קמחים" => Tag::Flour,
        "קפה וקפסולות קפה" => Tag::CoffeeAndCapsules,
        "קרקרים" => Tag::Crackers,
        "רטבי בישול" => Tag::CookingSauce,
        "רטבים לבישול" => Tag::CookingSauce,
        "ריבות וקונפיטורות" => Tag::Jam,
        "שוקולד" => Tag::Chocolate,
        "שימורי דגים" => Tag::CannedFish,
        "שימורי טונה ודגים" => Tag::CannedFish,
        "שימורי ירקות" => Tag::CannedVegetables,
        "שימורי עגבניות" => Tag::CannedTomatoes,
        "שמנים" => Tag::Oils,
        "שמפו ומרכך" => Tag::ShampooAndConditioner,
        "תבלינים" => Tag::Spices,
        "תה וחליטות" => Tag::TeaAndInfusions,
        "תחליפי גבינה" => Tag::CheeseAlternatives,
        "תכשירי גילוח ושעווה" => Tag::ShavingAndWaxingProducts,
        "תרכיזים" => Tag::Concentrates,
        // "אדום" => Tag::RED,
        // "גבינות צהובות ומותכות" => Tag::YELLOWANDHARDCHEESES,
        // "וופלים" => Tag::WAFFLES,
        // "ויטמינים, תוספי תזונה וצמחי מרפא" => Tag::VITAMINSNUTRITIONALSUPPLEMENTSHERBALREMEDIES,
        // "ופל, גביעים לגלידה" => Tag::ICECREAMCUPSANDCONES,
        // "טואלטיקה טבעית" => Tag::NATURALTOILETRIES,
        // "טיפוח הגוף והפנים" => Tag::BODYANDFACECARE,
        // "יוגורט פירות ותוספות" => Tag::FRUITYOGURTANDADDITIONS,
        // "לבן ורוזה" => Tag::WHITEANDROSE,
        // "למטבח" => Tag::FORTHEKITCHEN,
        // "מותגים מובילים השני בחצי" => Tag::LEADINGBRANDSSECONDHALF,
        // "מטליות" => Tag::TOWELS,
        // "נקניק ופסטרמה" => Tag::SAUSAGESANDPASTRAMI,
        // "סבון נוזלי ומוצרי רחצה" => Tag::LIQUIDSOAPANDTOILETRIES,
        // "סכו\"ם, קשיות, שיפודים" => Tag::WAXBROOMSMOPS,
        // "סלט חומוס וטחינה" => Tag::HUMMUSANDTAHINISALAD,
        // "עוגות וקינוחים בקירור" => Tag::COOLINGCAKESANDDESSERTS,
        // "פיצוחים ארוזים" => Tag::
        // "פיצוחים, אגוזים וגרעינים" => Tag::
        // "פיצוחים, אגוזים וגרעינים" => Tag::NUTSANDSEEDS,
        // "פריכיות ופתית" => Tag::CRUMBSANDCRISPS,
        // "צלחות, כוסות, קערות, מגשים" => Tag::
        // "צלחות, כוסות, קערות, מגשים" => Tag::PLATESCUPSBOWLSTRAYS,
        // "רצפות וכללי" => Tag::FLOORSANDGENERAL,
        // "תבניות אלומיניום,תבניות נייר" => Tag::ALUMINUMTRAYSPAPERTRAYS,
        // "תחליפי חלב" => Tag::DAIRYALTERNATIVES,
        _ => return None,
    })
}
