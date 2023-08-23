#[derive(Clone)]
pub enum Website {
    // Excalibur is a codename for various stores, including Victory,that use the same backend.
    // param: main_url
    Excalibur(&'static str),
    RamiLevy,
    Shufersal,
}

#[derive(Clone)]
pub struct OnlineStore {
    pub website: Website,
    pub name: &'static str,
}

impl OnlineStore {
    fn new(website: Website, name: &'static str) -> OnlineStore {
        OnlineStore { website, name }
    }
}

pub fn get_online_stores() -> Vec<OnlineStore> {
    [
        OnlineStore::new(
            Website::Excalibur("https://www.victoryonline.co.il/v2/retailers/1470"),
            "victory",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.ybitan.co.il/v2/retailers/1131"),
            "yenot_bitan",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.mega.co.il/v2/retailers/1182"),
            "mega",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.m2000.co.il/v2/retailers/1404"),
            "maayan_2000",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.ampm.co.il/v2/retailers/2"),
            "am_pm",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.tivtaam.co.il/v2/retailers/1062"),
            "tiv_taam",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.keshet-teamim.co.il/v2/retailers/1219"),
            "keshet",
        ),
        OnlineStore::new(
            Website::Excalibur("https://www.shukcity.co.il/v2/retailers/1254"),
            "shukcity",
        ),
        OnlineStore::new(Website::RamiLevy, "rami_levy"),
        OnlineStore::new(Website::Shufersal, "shufersal"),
    ]
    .to_vec()
}
