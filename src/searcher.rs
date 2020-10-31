use super::db::{Emoji, EmojiDb, Shortcodes};
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
        let shortcode_sets = self.db.shortcode_sets();

        self.db.emojis().filter_map(move |emoji| {
            emoji_contains_search(emoji, shortcode_sets, &search).map(|x| SearchResult {
                emoji: &emoji.emoji,
                matched_tag: x,
            })
        })
    }

    pub fn swap_db(&mut self, new_db: Rc<EmojiDb>) {
        self.db = new_db;
    }
}

fn emoji_contains_search<'a>(
    emoji: &'a Emoji,
    shortcode_sets: &'a [Shortcodes],
    search: &str,
) -> Option<&'a String> {
    for shortcode_set in shortcode_sets {
        let shortcode = shortcode_set.get(&emoji.hexcode);
        let matched_shortcode = match shortcode {
            Some(shortcodes) => shortcodes
                .iter()
                .find(|shortcode| shortcode.contains(search)),
            _ => None,
        };

        if let Some(s) = matched_shortcode {
            return Some(s);
        }
    }
    emoji.tags.iter().flatten().find(|tag| tag.contains(search))
}
