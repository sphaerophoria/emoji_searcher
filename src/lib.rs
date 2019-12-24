extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate app_dirs;
extern crate rmp_serde;
extern crate semver;

mod db;
mod searcher;

pub use db::EmojiDb;
pub use searcher::EmojiSearcher;
