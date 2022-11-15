use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref COUNTRY_TO_COUNTRY_CODE: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();

        map.insert("אפגניסטן", "AF");
        map.insert("איי אלנד", "AX");
        map.insert("אלבניה", "AL");
        map.insert("אלג'יריה", "DZ");
        map.insert("סמואה האמריקנית", "AS");
        map.insert("אנדורה", "AD");
        map.insert("אנגולה", "AO");
        map.insert("אנגווילה", "AI");
        map.insert("אנטארקטיקה", "AQ");
        map.insert("אנטיגואה וברבודה", "AG");
        map.insert("ארגנטינה", "AR");
        map.insert("ארמניה", "AM");
        map.insert("ארובה", "AW");
        map.insert("אוסטרליה", "AU");
        map.insert("אוסטריה", "AT");
        map.insert("אזרבייג'ן", "AZ");
        map.insert("אזורביג'אן", "AZ");
        map.insert("בהאמה", "BS");
        map.insert("בחריין", "BH");
        map.insert("בנגלדש", "BD");
        map.insert("ברבדוס", "BB");
        // Belarus
        map.insert("בלארוס", "BY");
        map.insert("בלרוס", "BY");
        map.insert("בלורוס", "BY");
        map.insert("בילרוס", "BY");
        map.insert("בלגיה", "BE");
        map.insert("בליז", "BZ");
        map.insert("בנין", "BJ");
        map.insert("ברמודה", "BM");
        map.insert("בהוטן", "BT");
        map.insert("בוליביה", "BO");
        map.insert("בוסניה והרצגובינה", "BA");
        map.insert("בוצואנה", "BW");
        map.insert("אי בובט", "BV");
        map.insert("ברזיל", "BR");
        map.insert("שטח האוקיאנוס ההודי הבריטי", "IO");
        map.insert("ברוניי דארוסלאם", "BN");
        map.insert("בולגריה", "BG");
        map.insert("בורקינה פאסו", "BF");
        map.insert("בורונדי", "BI");
        map.insert("קאבו ורדה", "CV");
        map.insert("קמבודיה", "KH");
        map.insert("קמרון", "CM");
        map.insert("קנדה", "CA");
        map.insert("איי קיימן", "KY");
        map.insert("הרפובליקה המרכז - אפריקאית", "CF");
        map.insert("צ'אד", "TD");
        // Chile
        map.insert("צ'ילה", "CL");
        map.insert("צילה", "CL");
        map.insert("סין", "CN");
        map.insert("China", "CN");
        map.insert("אי חג המולד", "CX");
        map.insert("איי קוקוס (קילינג)", "CC");
        map.insert("קולומביה", "CO");
        map.insert("קומורוס", "KM");
        map.insert("קונגו (הרפובליקה הדמוקרטית של)", "CD");
        map.insert("קונגו", "CG");
        map.insert("איי קוק", "CK");
        map.insert("קוסטה ריקה", "CR");
        map.insert("קוסטה ריקו", "CR");
        map.insert("Côte d'Ivoire", "CI");
        map.insert("קרואטיה", "HR");
        map.insert("קובה", "CU");
        map.insert("קוראסאו", "CW");
        map.insert("קַפרִיסִין", "CY");
        // CZ
        map.insert("צ'כיה", "CZ");
        map.insert("צכיה", "CZ");
        map.insert("דנמרק", "DK");
        map.insert("ג'יבוטי", "DJ");
        map.insert("דומיניקה", "DM");
        map.insert("הרפובליקה הדומיניקנית", "DO");
        map.insert("הרפובליקה הדומניקנית", "DO");
        map.insert("אקוודור", "EC");
        map.insert("מצרים", "EG");
        map.insert("אל סלבדור", "SV");
        map.insert("גיניאה המשוונית", "GQ");
        map.insert("אריתריאה", "ER");
        map.insert("אסטוניה", "EE");
        map.insert("Eswatini", "SZ");
        map.insert("אֶתִיוֹפִּיָה", "ET");
        map.insert("איי פוקלנד", "FK");
        map.insert("איי פרו", "FO");
        map.insert("פיג'י", "FJ");
        map.insert("פינלנד", "FI");
        map.insert("צרפת", "FR");
        map.insert("גיאנה הצרפתית", "GF");
        map.insert("פולינזיה הצרפתית", "PF");
        map.insert("שטחי דרום צרפתים", "TF");
        map.insert("גבון", "GA");
        map.insert("גמביה", "GM");
        // Multiple spellings for Georgia
        map.insert("ג'ורג'יה", "GE");
        map.insert("גאורגייה", "GE");
        map.insert("גאורגיה", "GE");
        map.insert("גרוזיה", "GE");
        map.insert("גרמניה", "DE");
        map.insert("גאנה", "GH");
        map.insert("גיברלטר", "GI");
        map.insert("יוון", "GR");
        map.insert("גרינלנד", "GL");
        map.insert("גרנדה", "GD");
        map.insert("גואם", "GU");
        map.insert("גואטמלה", "GT");
        map.insert("גוואטמלה", "GT");
        map.insert("גרנזי", "GG");
        map.insert("גינאה", "GN");
        map.insert("גינאה-ביסאו", "GW");
        map.insert("גיאנה", "GY");
        map.insert("האיטי", "HT");
        map.insert("שמע את איי ומקדונלד", "HM");
        map.insert("כורה קדושה", "VA");
        map.insert("הונדורס", "HN");
        map.insert("הונג קונג", "HK");
        map.insert("הונגריה", "HU");
        map.insert("איסלנד", "IS");
        map.insert("הודו", "IN");
        map.insert("אינדונזיה", "ID");
        map.insert("איראן", "IR");
        map.insert("עִירַאק", "IQ");
        map.insert("אירלנד", "IE");
        map.insert("האי מאן", "IM");
        map.insert("ישראל", "IL");
        map.insert("איטליה", "IT");
        map.insert("ג'מייקה", "JM");
        map.insert("יפן", "JP");
        map.insert("ג'רזי", "JE");
        map.insert("ירדן", "JO");
        map.insert("קזחסטן", "KZ");
        map.insert("קניה", "KE");
        map.insert("קנייה", "KE");
        map.insert("קיריבטי", "KI");
        map.insert("קוריאה הצפונית", "KP");
        // South Korea
        map.insert("קוריאה הדרומית", "KR");
        map.insert("דרום קוריאה", "KR");
        map.insert("דרום קוראה", "KR");
        map.insert("קוריאה", "KR");
        map.insert("כווית", "KW");
        map.insert("קירגיזסטן", "KG");
        map.insert("הרפובליקה הדמוקרטית של LAO", "LA");
        map.insert("לטביה", "LV");
        map.insert("לבנון", "LB");
        map.insert("לסוטו", "LS");
        map.insert("ליבריה", "LR");
        map.insert("לוב", "LY");
        map.insert("ליכטנשטיין", "LI");
        map.insert("ליטא", "LT");
        map.insert("ליטואניה", "LT");
        map.insert("לוקסמבורג", "LU");
        map.insert("מקאו", "MO");
        // Macedonia, with or without the "North"
        map.insert("צפון מקדוניה", "MK");
        map.insert("מקדוניה", "MK");
        map.insert("מדגסקר", "MG");
        map.insert("מלאווי", "MW");
        map.insert("מלזיה", "MY");
        map.insert("מלאזיה", "MY");
        map.insert("המלדיביים", "MV");
        map.insert("מלי", "ML");
        map.insert("מלטה", "MT");
        map.insert("איי מרשל", "MH");
        map.insert("מרטיניק", "MQ");
        map.insert("מאוריטניה", "MR");
        map.insert("מאוריציוס", "MU");
        map.insert("מיוט", "YT");
        // Mexico, with Kaf or Kof
        map.insert("מקסיקו", "MX");
        map.insert("מכסיקו", "MX");
        map.insert("מיקרונזיה", "FM");
        map.insert("מולדובה", "MD");
        map.insert("מונקו", "MC");
        map.insert("מונגוליה", "MN");
        map.insert("מונטנגרו", "ME");
        map.insert("מונטסראט", "MS");
        map.insert("מרוקו", "MA");
        map.insert("מוזמביק", "MZ");
        map.insert("מיאנמר", "MM");
        map.insert("נמיביה", "NA");
        map.insert("נאורו", "NR");
        map.insert("נפאל", "NP");
        map.insert("הולנד", "NL");
        map.insert("קלדוניה החדשה", "NC");
        map.insert("ניו זילנד", "NZ");
        map.insert("ניקרגואה", "NI");
        map.insert("ניז'ר", "NE");
        map.insert("ניגריה", "NG");
        map.insert("האי נורפולק", "NF");
        map.insert("איי צפון מריאנה", "MP");
        // Norway, with Beth or Vav
        map.insert("נורווגיה", "NO");
        map.insert("נורבגיה", "NO");
        map.insert("עומאן", "OM");
        map.insert("פקיסטן", "PK");
        map.insert("פקיסטאן", "PK");
        map.insert("פאלאו", "PW");
        map.insert("פלסטין", "PS");
        map.insert("רשות הפלסטינית", "PS");
        map.insert("פנמה", "PA");
        map.insert("פפואה גינאה החדשה", "PG");
        map.insert("פרגוואי", "PY");
        map.insert("פרו", "PE");
        map.insert("פיליפינים", "PH");
        map.insert("פיטקרן", "PN");
        map.insert("פולין", "PL");
        map.insert("פורטוגל", "PT");
        map.insert("פוארטו ריקו", "PR");
        map.insert("קטאר", "QA");
        map.insert("איחוד", "RE");
        map.insert("רומניה", "RO");
        map.insert("רוסיה", "RU");
        map.insert("רואנדה", "RW");
        map.insert("סנט ברתלמי", "BL");
        map.insert("סנט קיטס ונביס", "KN");
        map.insert("סנט לוסיה", "LC");
        map.insert("סנט פייר ומיקלון", "PM");
        map.insert("וינסנט הקדוש ו ה - גרנידיים", "VC");
        map.insert("סמואה", "WS");
        map.insert("סן מרינו", "SM");
        map.insert("סאו טומה ופרינסיפה", "ST");
        map.insert("ערב הסעודית", "SA");
        map.insert("סנגל", "SN");
        map.insert("סרביה", "RS");
        map.insert("סיישל", "SC");
        map.insert("סיירה לאונה", "SL");
        map.insert("סינגפור", "SG");
        map.insert("סינט מרטן (חלק הולנדי)", "SX");
        map.insert("סלובקיה", "SK");
        map.insert("סלובניה", "SI");
        map.insert("איי שלמה", "SB");
        map.insert("סומליה", "SO");
        map.insert("דרום אפריקה", "ZA");
        map.insert("דרום ג'ורג'יה ואיי סנדוויץ 'הדרום", "GS");
        map.insert("דרום סודן", "SS");
        map.insert("ספרד", "ES");
        map.insert("סרי לנקה", "LK");
        map.insert("סרילנקה", "LK");
        map.insert("סודן", "SD");
        map.insert("סורינאם", "SR");
        //  Sweden, with beth or vav
        map.insert("שוודיה", "SE");
        map.insert("שבדיה", "SE");
        // Switzerland, with one or two Yuds
        map.insert("שוויץ", "CH");
        map.insert("שווייץ", "CH");
        map.insert("הרפובליקה הערבית של סוריה", "SY");
        // Taiwan
        map.insert("טייוואן", "TW");
        map.insert("טאיוון", "TW");
        map.insert("טיוואן", "TW");
        map.insert("הרפובליקה הסינית (טאיוואן)", "TW");
        map.insert("טג'יקיסטן", "TJ");
        map.insert("טנזניה", "TZ");
        map.insert("תאילנד", "TH");
        map.insert("טימור-לסטה", "TL");
        map.insert("ללכת", "TG");
        map.insert("טוקלאו", "TK");
        map.insert("טונגה", "TO");
        map.insert("טרינידד וטובגו", "TT");
        map.insert("תוניסיה", "TN");
        // Two common names for Turkey
        map.insert("תורכיה", "TR");
        map.insert("טורקיה", "TR");
        map.insert("טורקמניסטן", "TM");
        map.insert("איי טורקס וקאיקוס", "TC");
        map.insert("טובלו", "TV");
        map.insert("אוגנדה", "UG");
        map.insert("אוקראינה", "UA");
        map.insert("איחוד האמירויות הערביות", "AE");
        map.insert("וד האמירויות הערביות", "AE");
        map.insert("איחוד האמירויות", "AE");
        map.insert("איחוד אימריות", "AE");
        map.insert("דובאי", "AE");
        // GB: England or Britain or United Kingdom + Scotland or London
        map.insert("סקוטלנד", "GB");
        map.insert("אנגליה", "GB");
        map.insert("בריטניה", "GB");
        map.insert("הממלכה המאוחדת", "GB");
        map.insert("לונדון", "GB");
        map.insert("ארצות הברית קטינה איים חיצוניים", "UM");
        // USA: Full name or acronym or Porto Rico
        map.insert("ארצות הברית", "US");
        map.insert("ארה\"ב", "US");
        map.insert("ארהב", "US");
        map.insert("פורטו ריקו", "US");
        map.insert("אורוגוואי", "UY");
        map.insert("אורוגואי", "UY");
        map.insert("אוזבקיסטן", "UZ");
        map.insert("ונואטו", "VU");
        map.insert("ונצואלה", "VE");
        // Vietnam
        map.insert("וייט נאם", "VN");
        map.insert("ויטנאם", "VN");
        map.insert("ויאטנם", "VN");
        map.insert("וייטנאם", "VN");
        map.insert("איי הבתולה (בריטים)", "VG");
        map.insert("וואליס ופוטונה", "WF");
        map.insert("סהרה המערבית", "EH");
        map.insert("תֵימָן", "YE");
        map.insert("זמביה", "ZM");
        map.insert("זימבבואה", "ZW");
        map
    };
}

pub fn to_country_code(s: &str) -> Option<&&str> {
    COUNTRY_TO_COUNTRY_CODE.get(s)
}