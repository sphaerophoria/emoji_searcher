use emoji_searcher::{EmojiDb, EmojiSearcher};
use std::error::Error;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn Error>> {
    let db = EmojiDb::new();
    let searcher = EmojiSearcher::new(Rc::new(db));
    let items = searcher.search(std::env::args().nth(1).unwrap());
    for item in items {
        print!("{}\n", item);
    }
    Ok(())
}
