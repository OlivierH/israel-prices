#[derive(Clone, Debug)]
pub enum Website {
    // param: username
    PublishedPrice(&'static str),
    // param: username, password
    PublishedPriceWithPassword(&'static str, &'static str),
    Shufersal,
    // These are different urls that work the same and use the same code.
    // Interistingly, the download prefix can be switched from one store to another
    // params: initial url, download prefix
    SimpleJsonToGet(&'static str, &'static str),
    // All MatrixCatalog stores use the same main page - filtering need to happen later.
    // Paramter: chain. currently: ויקטורי, מחסני השוק, ח. כהן, סופר ברקת
    MatrixCatalog(&'static str),
    // param: main page
    PublishPrice(&'static str),
    NetivHahesed,
    SuperPharm,
}

#[derive(Clone, Debug)]
pub struct Store {
    pub website: Website,
    pub name: &'static str,
}
impl Store {
    fn new(website: Website, name: &'static str) -> Store {
        Store { website, name }
    }
}

pub fn get_store_configs() -> Vec<Store> {
    // As defined and ordered on https://www.gov.il/he/departments/legalInfo/cpfta_prices_regulations
    [
        Store::new(
            Website::SimpleJsonToGet(
                "https://www.kingstore.co.il/Food_Law/MainIO_Hok.aspx",
                "https://www.kingstore.co.il/Food_Law/Download/",
            ),
            "king_store",
        ),
        Store::new(
            Website::SimpleJsonToGet(
                "http://maayan2000.binaprojects.com/MainIO_Hok.aspx",
                "http://maayan2000.binaprojects.com/Download/",
            ),
            "maayan_2000",
        ),
        Store::new(
            Website::SimpleJsonToGet(
                "https://goodpharm.binaprojects.com/MainIO_Hok.aspx",
                "https://goodpharm.binaprojects.com/Download/",
            ),
            "good_pharm",
        ),
        Store::new(Website::PublishedPrice("doralon"), "dor_alon"),
        Store::new(Website::MatrixCatalog("ויקטורי"), "victory"),
        Store::new(
            Website::SimpleJsonToGet(
                "http://zolvebegadol.binaprojects.com/MainIO_Hok.aspx",
                "http://zolvebegadol.binaprojects.com/Download/",
            ),
            "zol_vebegadol",
        ),
        Store::new(Website::MatrixCatalog("ח. כהן"), "h_cohen"),
        Store::new(Website::PublishedPrice("TivTaam"), "tiv_taam"),
        Store::new(
            Website::PublishPrice("http://publishprice.ybitan.co.il/"),
            "yenot_bitan",
        ),
        Store::new(
            Website::PublishPrice("http://publishprice.mega.co.il/"),
            "mega",
        ),
        Store::new(
            Website::PublishPrice("http://publishprice.mega-market.co.il/"),
            "mega_market",
        ),
        Store::new(Website::MatrixCatalog("מחסני השוק"), "mahsanei_hashuk"),
        Store::new(Website::PublishedPrice("HaziHinam"), "hatzi_hinam"),
        Store::new(Website::PublishedPrice("yohananof"), "yohananof"),
        Store::new(Website::PublishedPrice("osherad"), "osher_ad"),
        Store::new(Website::NetivHahesed, "netiv_hahesed"),
        Store::new(Website::PublishedPrice("SalachD"), "salach_dabbah"),
        // Store::new(
        //     Website::SimpleJsonToGet(
        //         "https://supersapir.binaprojects.com/Main.aspx",
        //         "https://supersapir.binaprojects.com/Download/",
        //     ),
        //     "super_sapir",
        // ),
        Store::new(Website::SuperPharm, "superpharm"),
        Store::new(Website::PublishedPrice("Stop_Market"), "stop_market"),
        Store::new(Website::MatrixCatalog("סופר ברקת"), "super_bareket"),
        Store::new(Website::PublishedPrice("politzer"), "politzer"),
        Store::new(
            Website::PublishedPriceWithPassword("Paz_bo", "paz468"),
            "paz",
        ),
        Store::new(
            Website::SimpleJsonToGet(
                "http://paz.binaprojects.com/MainIO_Hok.aspx",
                "http://paz.binaprojects.com/Download/",
            ),
            "super_yoda",
        ),
        Store::new(Website::PublishedPrice("freshmarket"), "fresh_market"),
        Store::new(Website::PublishedPrice("Keshet"), "keshet"),
        Store::new(Website::PublishedPrice("RamiLevi"), "rami_levy"),
        Store::new(Website::PublishedPrice("SuperCofixApp"), "super_cofix"),
        Store::new(Website::Shufersal, "shufersal"),
        Store::new(
            Website::SimpleJsonToGet(
                "http://shuk-hayir.binaprojects.com/MainIO_Hok.aspx",
                "http://shuk-hayir.binaprojects.com/Download/",
            ),
            "shuk_hayir", // AKA ShukCity
        ),
        Store::new(
            Website::SimpleJsonToGet(
                "http://shefabirkathashem.binaprojects.com/MainIO_Hok.aspx",
                "http://shefabirkathashem.binaprojects.com/Download/",
            ),
            "shefa_birkat_hashem",
        ),
    ]
    .to_vec()
}

const MINIMAL_STORE_CONFIGS: &'static [&'static str] =
    &["Yohananof", "Shufersal", "Super_Yoda", "Victory"];

pub fn get_minimal_store_configs() -> Vec<Store> {
    let mut results = Vec::new();

    for store in get_store_configs() {
        if MINIMAL_STORE_CONFIGS.contains(&store.name) {
            results.push(store);
        }
    }

    return results;
}

pub fn get_debug_store_configs() -> Vec<Store> {
    return get_store_configs()
        .into_iter()
        .filter(|s| s.name == "superpharm")
        .collect();
}

pub fn get_store_config(name: &str) -> Option<Store> {
    return get_store_configs().into_iter().find(|s| s.name == name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_non_empty() {
        for s in get_store_configs() {
            assert!(!s.name.is_empty());
        }
    }
}
