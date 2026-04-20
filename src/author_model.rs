use crate::library_model::{AuthorId, BookId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
// Base structure (object) for the rolodex.
// This owns all people (and, once implemented, organizations)
// it also holds various lists of labels used in people, etc.
pub struct AuthorModel {
    pub name: String,
    pub books: Vec<BookId>,
    pub appearences: Vec<BookId>,
    // this field is public so the library can use it directly
    // manipulating it other than through the library is liable
    // to produce run time errors
    pub akas: Vec<AuthorId>,
}

impl AuthorModel {
    // build an Author Model instance from the pieces
    pub fn build(
        name: &str,
        books: Vec<BookId>,
        appearences: Vec<BookId>,
        akas: Vec<AuthorId>,
    ) -> Self {
        Self {
            name: name.to_string(),
            books,
            appearences,
            akas,
        }
    }

    // change the name of an author
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    // get an iterator over the akas
    pub fn aka_iter(&self) -> impl Iterator<Item = &AuthorId> {
        self.akas.iter()
    }
}

// function to return an option of the position of a value in a vector
pub fn vec_find<T: std::cmp::PartialEq>(v: &[T], val: T) -> Option<usize> {
    let mut index = 0;
    for vecval in v.iter() {
        if *vecval == val {
            return Some(index);
        } else {
            index += 1;
        }
    }
    None
}
