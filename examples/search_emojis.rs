use emoji_searcher::{EmojiDb, EmojiSearcher};
use std::error::Error;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn Error>> {
    let db = EmojiDb::from_web()?;
    let searcher = EmojiSearcher::new(Rc::new(db));
    let items = searcher.search(std::env::args().nth(1).unwrap());
    for item in items {
        print!("{}\n", item.emoji);
    }
    Ok(())
}
