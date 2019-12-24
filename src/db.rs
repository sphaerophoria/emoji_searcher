use reqwest;
use rmp_serde;
use semver::Version;
use std::error::Error;
use std::io::{Read, Write};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

const EMOJIBASE_URL: &str = "https://cdn.jsdelivr.net/npm/emojibase-data@latest/en/data.json";
const EMOJIBASE_PACKAGE_JSON_URL: &str =
    "https://cdn.jsdelivr.net/npm/emojibase-data@latest/package.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Emoji {
    pub annotation: String, // CLDR34 localized description (primarily used for TTS)
    //pub name: String, // name according to official unicode data
    pub emoji: String,                   // actual emoji character
    pub shortcodes: Option<Vec<String>>, // community curated shortcodes (no surrounding colons)
    pub tags: Option<Vec<String>>,       // CLDR34 keywords
    pub skins: Option<Vec<Emoji>>,       // If there are skins
}

#[derive(Serialize, Deserialize)]
struct PackageJson {
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmojiDb {
    version: String,
    emojis: Vec<Emoji>,
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

        EmojiDb {
            version: local_package_json.version,
            emojis: local_data,
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

        Ok(EmojiDb {
            version: online_db_version.to_string(),
            emojis,
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
}
