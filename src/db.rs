use semver::Version;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::str::FromStr;

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
    "joypixels",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct Emoji {
    pub label: String, // CLDR34 localized description (primarily used for TTS)
    //pub name: String, // name according to official unicode data
    pub emoji: String,             // actual emoji character
    pub tags: Option<Vec<String>>, // CLDR34 keywords
    pub skins: Option<Vec<Emoji>>, // If there are skins
    pub hexcode: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DatabaseShortcode {
    Single(String),
    Multiple(Vec<String>),
}

impl DatabaseShortcode {
    fn into_vec(self) -> Vec<String> {
        match self {
            DatabaseShortcode::Single(s) => vec![s],
            DatabaseShortcode::Multiple(v) => v,
        }
    }
}

type DatabaseShortcodes = HashMap<String, DatabaseShortcode>;

pub type Shortcodes = HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize)]
struct PackageJson {
    version: String,
}

/// An emoji database
#[derive(Serialize, Deserialize)]
pub struct EmojiDb {
    version: String,
    emojis: Vec<Emoji>,
    shortcode_sets: Vec<Shortcodes>,
}

impl EmojiDb {
    /// Generates a new EmojiDb based off the database embedded in the library
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

        let mut shortcode_sets = Vec::new();
        for entry in shortcodes_archive.entries().unwrap() {
            let entry = entry.unwrap();

            let shortcode_set = parse_shortcode_database(entry)
                .expect("Failed to parse internal shortcode datatbase");

            shortcode_sets.push(shortcode_set);
        }

        EmojiDb {
            version: local_package_json.version,
            emojis: local_data,
            shortcode_sets,
        }
    }

    /// Generates an EmojiDb that was saved to the given cache
    pub fn from_cache<T: Read>(cache: &mut T) -> Result<EmojiDb, Box<dyn Error>> {
        // FIXME: test logic here
        Ok(rmp_serde::decode::from_read(cache)?)
    }

    fn get_online_version() -> Result<Version, Box<dyn Error>> {
        let package_json = reqwest::get(EMOJIBASE_PACKAGE_JSON_URL)?.json::<PackageJson>()?;

        Ok(Version::from_str(&package_json.version)?)
    }

    /// Generates a new EmojiDb based off data from the online repository maintained by the [Emojibase project](https://github.com/milesj/emojibase)
    pub fn from_web() -> Result<EmojiDb, Box<dyn Error>> {
        let online_db_version = Self::get_online_version()?;

        let emojis = reqwest::get(EMOJIBASE_URL).and_then(|mut data| data.json::<Vec<Emoji>>())?;

        let mut shortcode_sets = Vec::new();
        for shortcode_source in &SHORTCODE_SOURCES {
            let shortcode_set_data = reqwest::get(&format!(
                "{}/{}.json",
                EMOJIBASE_SHORTCODE_DIRECTORY_URL, shortcode_source
            ))?;

            let shortcode_set = parse_shortcode_database(shortcode_set_data)?;

            shortcode_sets.push(shortcode_set);
        }

        Ok(EmojiDb {
            version: online_db_version.to_string(),
            emojis,
            shortcode_sets,
        })
    }

    /// Saves the existing EmojiDb the provided cache
    pub fn save<T: Write>(&self, cache: &mut T) -> Result<(), Box<dyn Error>> {
        // FIXME: test save/read logic
        rmp_serde::encode::write(cache, self)?;
        Ok(())
    }

    /// Checks if a new version of the [Emojibase project](https://github.com/milesj/emojibase) is available
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

    /// Retrieves an iterator over all emojis in the db
    pub fn emojis(&self) -> impl Iterator<Item = &Emoji> {
        self.emojis.iter()
    }

    /// Retrieves the corresponding [Emojibase](https://github.com/milesj/emojibase) version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Retrieves all shortcodes
    pub fn shortcode_sets(&self) -> &[Shortcodes] {
        &self.shortcode_sets
    }
}

impl Default for EmojiDb {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_shortcode_database<R: Read>(source: R) -> Result<Shortcodes, Box<dyn Error>> {
    let database_shortcode_set: DatabaseShortcodes = serde_json::from_reader(source)?;

    let shortcode_set = database_shortcode_set
        .into_iter()
        .map(|(k, v)| (k, v.into_vec()))
        .collect();

    Ok(shortcode_set)
}
