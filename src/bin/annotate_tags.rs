use std::collections::HashMap;

use anyhow::Result;
use israel_prices::{
    models::{ItemInfo, ItemKey},
    sqlite_utils::save_tags_to_sqlite,
    tags::Tag,
};
use itertools::Itertools;
use multimap::MultiMap;
use tracing::info;
use tracing_subscriber::prelude::*;

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
        scraped_data
            .iter_all()
            .map(|(_key, vec)| vec.len())
            .sum::<usize>()
    );

    let mut numitemswithtags = 0;
    let numitems = item_infos.len();

    let mut tags: MultiMap<ItemKey, Tag> = MultiMap::new();

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
                if let Some(tag) = tag {
                    hastag = true;

                    if tags
                        .get_vec(&itemkey)
                        .map(|v| !v.contains(&tag))
                        .unwrap_or(true)
                    {
                        tags.insert(itemkey, tag);
                    }
                }
            }
        }
        if hastag {
            numitemswithtags += 1;
        } else {
            for category_vec in &category_vecs {
                if let Some(category) = category_vec.0.last() {
                    // println!("!{}-{category}!", category_vec.1);
                    // if category ==  {
                    //     println!("!{:?}!", category_vec.0);
                    // }
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
    // println!("{:?}", tags);
    save_tags_to_sqlite(&tags)?;
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
        "אדום" => Tag::RedWine,
        "אורז, קוסקוס ופתיתים" => Tag::RiceCouscousAndPasta,
        "אלכוהול" => Tag::Alcohol,
        "אסלות ואמבטיות" => Tag::ToiletAndBathCleaningProducts,
        "בונבוניירה ופרלינים" => Tag::BonbonsAndPralines,
        "ביסקוויטים" => Tag::Biscuits,
        "בירה" => Tag::Beer,
        "בצקים" => Tag::Dough,
        "בקבוקים" => Tag::Bottles,
        "בשמים לגברים" => Tag::MenPerfume,
        "בשמים לנשים" => Tag::WomenPerfume,
        "בשר קפוא" => Tag::FrozenMeat,
        "גבינות מלוחות" => Tag::SaltedCheeses,
        "גבינות מעדניה" => Tag::CheeseByWeight,
        "גבינות קשות / חצי קשות" => Tag::HardOrSemiHardCheese,
        "גבינות שמנת" => Tag::CreamyCheeses,
        "גבינות" => Tag::Cheese,
        "גלידות אישיות" => Tag::IndividualIceCream,
        "גלידות משפחתיות" => Tag::FamilyIceCreams,
        "גרנולות ומוזלי" => Tag::GranolaAndCereals,
        "דאודורנט" => Tag::Deodorant,
        "דאודורנטים" => Tag::Deodorant,
        "דגים קפואים" => Tag::FrozenFish,
        "דגני בוקר" => Tag::BreakfastCereals,
        "וויסקי" => Tag::Whisky,
        "וופלים" => Tag::Waffles,
        "ויטמינים, תוספי תזונה וצמחי מרפא" => {
            Tag::VitaminsAndDietarySupplements
        }
        "ויטמינים" => Tag::Vitamins,
        "זיתים" => Tag::Olives,
        "חומוס/טחינה" => Tag::HummusAndTahini,
        "חטיפי בריאות" => Tag::HealthSnacks,
        "חטיפי דגנים ואנרגיה" => Tag::EnergyAndCerealSnacks,
        "חטיפי שוקולד" => Tag::ChocolateSnacks,
        "חטיפים ארוזים" => Tag::PackagedSnacks,
        "חטיפים מלוחים" => Tag::SaltySnacks,
        "חלב" => Tag::Milk,
        "טבלאות שוקולד" => Tag::ChocolateTablets,
        "טואלטיקה טבעית" => Tag::NaturalToiletries,
        "טיפוח הגוף והפנים" => Tag::FaceAndBodyCare,
        "טיפוח עור" => Tag::SkinCare,
        "יוגורט לבן" => Tag::PlainYogurt,
        "יוגורט" => Tag::Yogurt,
        "יין" => Tag::Wine,
        "ירקות קפואים" => Tag::FrozenVegetables,
        "ירקות" => Tag::Vegetables,
        "כביסה" => Tag::Laundry,
        "כלי מטבח" => Tag::KitchenUtensils,
        "כלים חד פעמיים" => Tag::DisposableDishes,
        "לבן ורוזה" => Tag::WhiteAndRoseWine,
        "לחם" => Tag::Bread,
        "לחמניות ופיתות" => Tag::RollsAndPitas,
        "למטבח" => Tag::ForTheKitchen,
        "מאגדות גלידה" => Tag::IceCreamTubs,
        "מברשות שיניים" => Tag::Tootbrushes,
        "מוצרי סויה וצמחוני" => Tag::SoyAndPlantBasedProducts,
        "מוצרים ואביזרים לשיער" => Tag::HairProductsAndAccessories,
        "מזון לבעלי חיים" => Tag::PetFood,
        "מזון מוכן לבישול" => Tag::ReadyToCook,
        "מזון תינוקות" => Tag::BabyFood,
        "מטהר אוויר" => Tag::AirPurifier,
        "מיצים ונקטרים" => Tag::JuicesAndNectars,
        "מיצים" => Tag::Juices,
        "ממרחים ומתבלים" => Tag::JamsAndDips,
        "ממרחים מתוקים" => Tag::SweetSpreads,
        "ממרחים" => Tag::Spreads,
        "מנות להכנה מהירה" => Tag::QuickPrepMeals,
        "מסטיקים" => Tag::Gum,
        "מעדניית גבינות" => Tag::CheeseByWeight,
        "מעדנים" => Tag::Maadanim,
        "מרכך כביסה" => Tag::LaundrySoftener,
        "מרקים ומזון מהיר להכנה" => Tag::SoupsAndQuickPrepFood,
        "מרקים" => Tag::Soups,
        "משחות שיניים" => Tag::DentalPaste,
        "משקאות חלב ושוקו" => Tag::MilkAndChocolateDrinks,
        "משקאות חלב" => Tag::DairyDrinks,
        "משקאות מוגזים" => Tag::CarbonatedDrinks,
        "משקאות קלים, מים בטעמים" => Tag::SoftDrinksFlavoredWater,
        "משקאות קלים" => Tag::SoftDrinks,
        "ניקוי כלים ולמדיח" => Tag::DishwashingAndDetergents,
        "ניקוי כלים ומטבח" => Tag::KitchenAndDishCleaning,
        "ניקוי לבית" => Tag::HouseCleaning,
        "נקטרים" => Tag::Nectars,
        "נקטרים/מיצים" => Tag::NectarsAndJuices,
        "נקניק ופסטרמה" => Tag::SausageAndPastrami,
        "נקניקים ארוזים" => Tag::PackedSausages,
        "נרות וגפרורים" => Tag::CandlesAndMatches,
        "סבונים" => Tag::Soap,
        "סוכריות" => Tag::Candy,
        "סיגריות" => Tag::Cigarettes,
        "סלט חומוס וטחינה" => Tag::HummusAndTahiniSalad,
        "סלטים שונים" => Tag::PreparedSalads,
        "עוגות ארוזות" => Tag::PackagedCakes,
        "עוגות וקינוחים בקירור" => Tag::CakesAndDessertsInCooling,
        "עוגיות ארוזות" => Tag::CookiesPackaged,
        "עוגיות וביסקוויטים" => Tag::CookiesAndBiscuits,
        "עוגיות" => Tag::Cookies,
        "עוף טרי" => Tag::FreshChicken,
        "עלים ועשבי תיבול" => Tag::LeavesAndHerbs,
        "פיצוחים ארוזים" => Tag::PackagedPitsuhim,
        "פיצוחים, אגוזים וגרעינים" => Tag::Pitsuhim,
        "פירות" => Tag::Fruit,
        "פסטה,אטריות ונודלס" => Tag::Pasta,
        "פסטות" => Tag::Pasta,
        "פריכיות ופתית" => Tag::RiceCrispsAndCrackers,
        "פרכיות ופתית" => Tag::RiceCrispsAndCrackers,
        "צלחות חד פעמי" => Tag::DisposablePlates,
        "קוסמטיקה טבעית" => Tag::NaturalCosmetics,
        "קטניות/דגנים" => Tag::CerealsAndGrains,
        "קטשופ, מיונז, חרדל, טחינה" => Tag::KetchupMayonnaiseMustardTahini,
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
        "שפתונים" => Tag::Lipsticks,
        "תבלינים" => Tag::Spices,
        "תה וחליטות" => Tag::TeaAndInfusions,
        "תחבושות היגיינה" => Tag::Pads,
        "תחליפי גבינה" => Tag::CheeseAlternatives,
        "תחליפי חלב" => Tag::MilkSubstitutes,
        "תכשירי גילוח ושעווה" => Tag::ShavingAndWaxingProducts,
        "תערובות להכנה מהירה" => Tag::QuickPrepMixes,
        "תרכיזים" => Tag::Concentrates,
        // "גבינות צהובות ומותכות" => Tag::YELLOWANDHARDCHEESES,
        // "ויטמינים, תוספי תזונה וצמחי מרפא" => Tag::VITAMINSNUTRITIONALSUPPLEMENTSHERBALREMEDIES,
        // "ופל, גביעים לגלידה" => Tag::ICECREAMCUPSANDCONES,
        // "טיפוח הגוף והפנים" => Tag::BODYANDFACECARE,
        // "יוגורט פירות ותוספות" => Tag::FRUITYOGURTANDADDITIONS,
        // "למטבח" => Tag::FORTHEKITCHEN,
        // "מותגים מובילים השני בחצי" => Tag::LEADINGBRANDSSECONDHALF,
        // "מטליות" => Tag::TOWELS,
        // "נקניק ופסטרמה" => Tag::SAUSAGESANDPASTRAMI,
        // "סבון נוזלי ומוצרי רחצה" => Tag::LIQUIDSOAPANDTOILETRIES,
        // "סכו\"ם, קשיות, שיפודים" => Tag::WAXBROOMSMOPS,
        // "סלט חומוס וטחינה" => Tag::HUMMUSANDTAHINISALAD,
        // "ספורטאים" => Tag::Sports,
        // "עולם התינוקות והילדים" => Tag::WorldOfBabiesAndChildren,
        // "פיצוחים, אגוזים וגרעינים" => Tag::
        // "פריכיות ופתית" => Tag::CRUMBSANDCRISPS,
        // "צלחות, כוסות, קערות, מגשים" => Tag::PlatesCupsBowlsAndTrays,
        // "רצפות וכללי" => Tag::FLOORSANDGENERAL,
        // "שוקולד, וופלים גביעי גלידה וסוכריות" => Tag::ChocolateWafersIceCreamCupsCandies
        // "תבניות אלומיניום,תבניות נייר" => Tag::ALUMINUMTRAYSPAPERTRAYS,
        // "תחליפי חלב" => Tag::DAIRYALTERNATIVES,
        _ => return None,
    })
}
