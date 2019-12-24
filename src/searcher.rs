use super::db::{Emoji, EmojiDb};
use std::rc::Rc;

pub struct EmojiSearcher {
    db: Rc<EmojiDb>,
}

pub struct SearchResult<'a> {
    pub emoji: &'a String,
    pub matched_tag: &'a String,
}

impl EmojiSearcher {
    pub fn new(db: Rc<EmojiDb>) -> EmojiSearcher {
        EmojiSearcher { db }
    }

    // FIXME: Sort based on some form of score
    pub fn search(&self, search: String) -> impl Iterator<Item = SearchResult> {
        self.db.emojis().filter_map(move |emoji| {
            emoji_contains_search(emoji, &search).map(|x| SearchResult {
                emoji: &emoji.emoji,
                matched_tag: x,
            })
        })
    }

    pub fn swap_db(&mut self, new_db: Rc<EmojiDb>) {
        self.db = new_db;
    }
}

fn emoji_contains_search<'a>(emoji: &'a Emoji, search: &str) -> Option<&'a String> {
    if emoji.shortcodes.is_none() {
        return None;
    }

    let shortcodes = emoji.shortcodes.as_ref().unwrap();
    let tags = emoji.tags.as_ref().unwrap();

    shortcodes
        .iter()
        .chain(tags)
        .find(|shortcode| shortcode.contains(search))
}
