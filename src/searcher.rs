use super::db::{Emoji, EmojiDb, Shortcodes};
use std::rc::Rc;

/// An emoji searcher
pub struct EmojiSearcher {
    db: Rc<EmojiDb>,
}

pub struct SearchResult<'a> {
    pub emoji: &'a String,
    pub matched_tag: &'a String,
}

impl EmojiSearcher {
    /// Creates a new emoji searcher with the provided database
    pub fn new(db: Rc<EmojiDb>) -> EmojiSearcher {
        EmojiSearcher { db }
    }

    /// Search for an emoji matching the given string.
    ///
    /// This will match any emoji that has a shortcode or a tag that contains the
    /// provided search string. The return value is an iterator of results that
    /// contain both the provided emoji as well as the tag that matched the
    /// provided search
    pub fn search(&self, search: String) -> impl Iterator<Item = SearchResult> {
        // TODO: Sort based on some form of score
        let shortcode_sets = self.db.shortcode_sets();

        self.db.emojis().filter_map(move |emoji| {
            emoji_contains_search(emoji, shortcode_sets, &search).map(|x| SearchResult {
                emoji: &emoji.emoji,
                matched_tag: x,
            })
        })
    }

    /// Update the internal emoji database with a new one. Can be used with
    /// [EmojiDb::from_web](struct.EmojiDb.html#method.from_web) to provide an updated database
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
