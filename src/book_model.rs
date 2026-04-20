use crate::library_model::AuthorId;
use crate::library_window::book_gui::BookGuiData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
// Base structure (object) for the rolodex.
// This owns all people (and, once implemented, organizations)
// it also holds various lists of labels used in people, etc.
pub struct BookModel {
    pub book: ItemInfo,
    pub series: String,
    pub in_series: String,
    pub publisher: String,
    pub isbn: String,
    pub pubdate: String,
    pub genre: String,
    pub format: String,
    pub annotation: String,
    pub content_type: ContentType,
    pub contents: Vec<ItemInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContentType {
    Singular,
    Collection,
    Anthology,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemInfo {
    pub title: String,
    pub authors: Vec<AuthorId>,
}

impl BookModel {
    // create a book from the gui information holder
    pub fn from_bookgui(bgs: &BookGuiData) -> BookModel {
        let bii = ItemInfo {
            title: bgs.book.title.clone(),
            authors: bgs.book.authors.clone(),
        };
        let contents = bgs
            .contents
            .iter()
            .map(|bih| ItemInfo {
                title: bih.title.clone(),
                authors: bih.authors.clone(),
            })
            .collect::<Vec<_>>();
        let content_type = if !bgs.is_collection {
            ContentType::Singular
        } else if !bgs.is_anthology {
            ContentType::Collection
        } else {
            ContentType::Anthology
        };
        Self {
            book: bii.clone(),
            series: bgs.series.clone(),
            in_series: bgs.in_series.clone(),
            publisher: bgs.publisher.clone(),
            isbn: bgs.isbn.clone(),
            pubdate: bgs.pubdate.clone(),
            genre: bgs.genre.clone(),
            format: bgs.format.clone(),
            annotation: bgs.annotation.text(),
            content_type,
            contents: contents.clone(),
        }
    }

    // extract the title of a book
    pub fn title(&self) -> &str {
        &self.book.title
    }

    // get an iterator over the ids of authors of a book
    pub fn author_iter(&self) -> impl Iterator<Item = &AuthorId> {
        self.book.authors.iter()
    }
}
