use std::collections::HashMap;
use std::str::FromStr;
use std::string::ParseError;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum FileType {
    PriceFull,
    StoresFull,
    PromoFull,
    Price,
    Promo,
    Ignored,
}
impl FileType {
    fn extract_prefix(name: &str) -> (FileType, &str) {
        if name.starts_with("PriceFull") {
            return (FileType::PriceFull, "PriceFull");
        }
        if name.starts_with("pricefull") {
            return (FileType::PriceFull, "pricefull");
        }
        if name.starts_with("StoresFull") {
            return (FileType::StoresFull, "StoresFull");
        }
        if name.starts_with("storesfull") {
            return (FileType::StoresFull, "storesfull");
        }
        if name.starts_with("Stores") {
            return (FileType::StoresFull, "Stores");
        }
        if name.starts_with("PromoFull") {
            return (FileType::PromoFull, "PromoFull");
        }
        if name.starts_with("Price") {
            return (FileType::Price, "Price");
        }
        if name.starts_with("price") {
            return (FileType::Price, "price");
        }
        if name.starts_with("Promo") {
            return (FileType::Promo, "Promo");
        }
        if name.starts_with("promo") {
            return (FileType::Promo, "promo");
        }
        if name.starts_with("NULL")
            || name.starts_with("created")
            || name.starts_with("Events")
            || name.starts_with("New")
            || name.ends_with(".jpg")
            || name.is_empty()
        {
            return (FileType::Ignored, "");
        }
        // TODO: in development, panic, but in prod, return "Ignored" everytime.
        panic!("Unparseable file: '{name}'")
    }
    fn is_interesting(self: &Self) -> bool {
        return FileType::PriceFull == *self || *self == FileType::StoresFull;
    }
}

#[derive(Debug)]
pub struct FileInfo {
    file_type: FileType,
    chain: String,
    store: String,
    date: String,
    pub filename: String,
    pub source: String,
    pub cookie: String, // needed only when using a different cookie per file.
}

impl FileInfo {
    pub fn is_interesting(self: &Self) -> bool {
        return self.file_type.is_interesting();
    }

    pub fn with_source(self: Self, source: &str) -> Self {
        FileInfo {
            source: source.to_string(),
            ..self
        }
    }
    pub fn with_cookie(self: Self, cookie: &str) -> Self {
        FileInfo {
            cookie: cookie.to_string(),
            ..self
        }
    }

    pub fn key(self: &Self) -> (FileType, String, String) {
        return (self.file_type, self.chain.clone(), self.store.clone());
    }

    pub fn from_str_iter(
        vals: impl Iterator<Item = String>,
        file_limit: Option<usize>,
    ) -> std::vec::IntoIter<FileInfo> {
        FileInfo::keep_most_recents(
            vals.map(|link| link.parse::<FileInfo>().unwrap())
                .filter(|fi| fi.is_interesting())
                .collect(),
            file_limit,
        )
        .into_iter()
    }

    pub fn keep_most_recents(data: Vec<FileInfo>, file_limit: Option<usize>) -> Vec<FileInfo> {
        let mut non_stores: HashMap<(FileType, String, String), FileInfo> = HashMap::new();
        let mut stores: HashMap<(FileType, String, String), FileInfo> = HashMap::new();

        for file_info in data {
            let map = match file_info.file_type {
                FileType::StoresFull => &mut stores,
                _ => &mut non_stores,
            };
            if !map.contains_key(&file_info.key())
                || map.get(&file_info.key()).unwrap().date < file_info.date
            {
                map.insert(file_info.key(), file_info);
            }
        }

        let mut recents: Vec<FileInfo> = stores.into_values().collect::<Vec<FileInfo>>();
        let recents_non_stores: Vec<FileInfo> = match file_limit {
            Some(i) => non_stores.into_values().take(i).collect::<Vec<FileInfo>>(),
            None => non_stores.into_values().collect::<Vec<FileInfo>>(),
        };
        recents.extend(recents_non_stores);
        recents
    }
}

fn extract_filename(url: &str) -> &str {
    let url = {
        if !url.contains("/") {
            url
        } else {
            url.rsplit_once("/").unwrap().1
        }
    };
    let url = {
        if !url.contains("?") {
            url
        } else {
            url.split_once("?").unwrap().0
        }
    };
    return url;
}

impl FromStr for FileInfo {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let filename = extract_filename(s);
        let (file_type, prefix) = FileType::extract_prefix(filename);

        if file_type == FileType::Ignored {
            return Ok(FileInfo {
                file_type: file_type,
                chain: "".to_string(),
                store: "".to_string(),
                date: "".to_string(),
                filename: "".to_string(),
                source: "".to_string(),
                cookie: "".to_string(),
            });
        }

        let mut parts: Vec<String> = filename
            .strip_prefix(prefix)
            .and_then(|s| s.split_once("."))
            .map(|s| s.0)
            .map(|s| s.split("-").map(|s| s.to_string()).collect())
            .expect(format!("Error parsing {}", s).as_str());
        while parts.len() > 3 {
            parts.pop();
        }

        if file_type == FileType::StoresFull && parts.len() == 2 {
            Ok(FileInfo {
                file_type: file_type,
                date: parts.pop().unwrap(),
                store: "".to_string(),
                chain: parts.pop().unwrap(),
                filename: String::from(filename),
                source: String::from(s),
                cookie: "".to_string(),
            })
        } else {
            assert_eq!(parts.len(), 3, "Error handling {}", s);

            Ok(FileInfo {
                file_type: file_type,
                date: parts.pop().unwrap(),
                store: parts.pop().unwrap(),
                chain: parts.pop().unwrap(),
                filename: String::from(filename),
                source: String::from(s),
                cookie: "".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prices_full() {
        let f: FileInfo = "Price7290058179875-040-202210261508.gz".parse().unwrap();
        assert!(!f.is_interesting());
    }
}
