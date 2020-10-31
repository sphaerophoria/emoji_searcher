use reqwest;
use rmp_serde;
use semver::Version;
use std::error::Error;
use std::io::{Read, Write};
use std::str::FromStr;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const EMOJIBASE_URL: &str = "https://cdn.jsdelivr.net/npm/emojibase-data@latest/en/data.json";
const EMOJIBASE_PACKAGE_JSON_URL: &str =
    "https://cdn.jsdelivr.net/npm/emojibase-data@latest/package.json";
const EMOJIBASE_SHORTCODE_DIRECTORY_URL: &str =
    "https://cdn.jsdelivr.net/npm/emojibase-data@latest/en/shortcodes";

const SHORTCODE_SOURCES: [&str; 6] = [
    "cldr",
    "emojibase",
    "emojibase-legacy",
    "github",
    "iamcal",
    "joypixels"
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Emoji {
    pub annotation: String, // CLDR34 localized description (primarily used for TTS)
    //pub name: String, // name according to official unicode data
    pub emoji: String,                   // actual emoji character
    pub tags: Option<Vec<String>>,       // CLDR34 keywords
    pub skins: Option<Vec<Emoji>>,       // If there are skins
    pub hexcode: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShortcodeType {
    Single(String),
    Multiple(Vec<String>),
}

pub type Shortcodes = HashMap<String, ShortcodeType>;

#[derive(Serialize, Deserialize)]
struct PackageJson {
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmojiDb {
    version: String,
    emojis: Vec<Emoji>,
    shortcodes: Vec<Shortcodes>,
}

impl EmojiDb {
    pub fn new() -> EmojiDb {
        let local_data_bytes =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/data.json"));
        let local_package_json_bytes =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/package.json"));

        let local_package_json: PackageJson =
            serde_json::from_slice(local_package_json_bytes).unwrap();
        let local_data = serde_json::from_slice(local_data_bytes).unwrap();

        let local_shortcodes_tar_bytes =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/shortcodes.tar"));

        let mut shortcodes_archive = tar::Archive::new(local_shortcodes_tar_bytes.as_ref());

        let mut shortcodes = Vec::new();
        for entry in shortcodes_archive.entries().unwrap() {
            let entry = entry.unwrap();
            let shortcode_set: Shortcodes = serde_json::from_reader(entry).unwrap();
            shortcodes.push(shortcode_set);
        }

        EmojiDb {
            version: local_package_json.version,
            emojis: local_data,
            shortcodes: shortcodes,
        }
    }

    // FIXME: test logic here
    pub fn from_cache<T: Read>(cache: &mut T) -> Result<EmojiDb, Box<dyn Error>> {
        Ok(rmp_serde::decode::from_read(cache)?)
    }

    fn get_online_version() -> Result<Version, Box<dyn Error>> {
        let package_json = reqwest::get(EMOJIBASE_PACKAGE_JSON_URL)?.json::<PackageJson>()?;

        Ok(Version::from_str(&package_json.version)?)
    }

    pub fn from_web() -> Result<EmojiDb, Box<dyn Error>> {
        let online_db_version = Self::get_online_version()?;

        let emojis = reqwest::get(EMOJIBASE_URL).and_then(|mut data| data.json::<Vec<Emoji>>())?;

        let mut shortcodes = vec![];
        for shortcode_source in &SHORTCODE_SOURCES {
            let mut source_shortcodes_data = reqwest::get(&format!("{}/{}.json", EMOJIBASE_SHORTCODE_DIRECTORY_URL, shortcode_source))?;
            let source_shortcodes = source_shortcodes_data.json::<Shortcodes>()?;
            shortcodes.push(source_shortcodes);
        }

        Ok(EmojiDb {
            version: online_db_version.to_string(),
            emojis,
            shortcodes,
        })
    }

    // FIXME: test save/read logic
    pub fn save<T: Write>(&self, cache: &mut T) -> Result<(), Box<dyn Error>> {
        rmp_serde::encode::write(cache, self)?;
        Ok(())
    }

    pub fn needs_update(&self) -> bool {
        let current_db_version = match Version::from_str(&self.version) {
            Ok(x) => x,
            Err(_) => {
                error!("Cannot parse current emoji database version");
                return true;
            }
        };

        let online_db_version = match Self::get_online_version() {
            Ok(x) => x,
            Err(_) => {
                warn!("Online emoji database verison not found");
                return false;
            }
        };

        current_db_version < online_db_version
    }

    pub fn emojis(&self) -> impl Iterator<Item = &Emoji> {
        self.emojis.iter()
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn shortcodes(&self) -> &[Shortcodes] {
        &self.shortcodes
    }
}
