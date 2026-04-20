use crate::author_model::{AuthorModel, vec_find};
use crate::book_model::BookModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
// Base structure (object) for the rolodex.
// This owns all people (and, once implemented, organizations)
// it also holds various lists of labels used in people, etc.
//
// Ntoe that the library is responsible for enforcing the consistency
// constraint on aka clusters.  All members of a cluster must have
// akas entries for all other members of the cluster
// inconsistent structure won't work
pub struct LibraryModel {
    name: String,
    next_book_id: BookId,
    books: HashMap<BookId, BookModel>,
    next_author_id: AuthorId,
    authors: HashMap<AuthorId, AuthorModel>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
// Type wrapper for Book Ids
pub struct BookId(usize);

impl BookId {
    // create an initial (minimal) BookId
    pub fn start() -> Self {
        Self(0)
    }

    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
// Type wrapper for Author IDs
pub struct AuthorId(usize);

impl AuthorId {
    // create an initial (minimal) AuthorID
    pub fn start() -> Self {
        Self(0)
    }

    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

impl LibraryModel {
    // Create an empty library with the given name
    pub fn new(name: &str) -> Self {
        let mut lib = Self {
            name: name.to_string(),
            next_author_id: AuthorId::start(),
            next_book_id: BookId::start(),
            books: HashMap::new(),
            authors: HashMap::new(),
        };
        let anon = AuthorModel::build("Anonymous", Vec::new(), Vec::new(), Vec::new());
        let unknown = AuthorModel::build("Unknown", Vec::new(), Vec::new(), Vec::new());
        let anon_id = lib.add_author(&anon);
        let unknown_id = lib.add_author(&unknown);
        lib.add_aka_pair(anon_id, unknown_id);
        lib
    }

    // get the name of this library
    pub fn name(&self) -> &str {
        &self.name
    }

    // set the name of this library
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    // get an Option Reference to a book by a BookId
    pub fn get_book(&self, id: BookId) -> Option<&BookModel> {
        self.books.get(&id)
    }

    // add a new book
    pub fn add_book(&mut self, book: &BookModel) -> BookId {
        let bookid = self.next_book_id;
        self.books.insert(bookid, book.clone());
        self.next_book_id = self.next_book_id.increment();
        bookid
    }

    // replace the book with a given ID
    pub fn replace_book(&mut self, id: BookId, book: &BookModel) {
        self.books.insert(id, book.clone());
    }

    // delete a book
    pub fn delete_book(&mut self, bid: BookId) {
        self.books.remove(&bid);
    }

    // get an iterator over the author IDs
    pub fn book_iter(&self) -> impl Iterator<Item = &BookId> {
        self.books.keys()
    }

    // get an Option Reference to an author by an AuthorId
    pub fn get_author(&self, id: &AuthorId) -> Option<&AuthorModel> {
        self.authors.get(id)
    }

    // get a mutable (option of a) reference to an Author
    pub fn mut_author(&mut self, id: AuthorId) -> Option<&mut AuthorModel> {
        self.authors.get_mut(&id)
    }

    // add a new author
    pub fn add_author(&mut self, author: &AuthorModel) -> AuthorId {
        let authorid = self.next_author_id;
        self.authors.insert(authorid, author.clone());
        self.next_author_id = self.next_author_id.increment();
        authorid
    }

    // check that there are no references to a given author in the library
    pub fn check_auth_noref(&self, aid: AuthorId) -> bool {
        if let Some(auth) = self.authors.get(&aid) {
            auth.books.is_empty() && auth.appearences.is_empty() && auth.akas.is_empty()
        } else {
            true
        }
    }

    // delete an author entry
    // assumes that check_auth_noref has been called
    pub fn delete_author(&mut self, aid: AuthorId) {
        self.authors.remove(&aid);
    }

    // get an iterator over the author IDs
    pub fn author_iter(&self) -> impl Iterator<Item = &AuthorId> {
        self.authors.keys()
    }

    // add a match pair of aka relationships based on a pair of author IDs
    fn add_aka_pair(&mut self, aid1: AuthorId, aid2: AuthorId) {
        let auth = self.authors.get(&aid1);
        if auth.is_none() || auth.unwrap().akas.contains(&aid2) {
            return;
        }
        let auth = self.authors.get(&aid2);
        if auth.is_none() || auth.unwrap().akas.contains(&aid1) {
            return;
        }
        let auth = self.authors.get_mut(&aid1).unwrap();
        auth.akas.push(aid2);
        let auth = self.authors.get_mut(&aid2).unwrap();
        auth.akas.push(aid1);
    }

    //remvoe a matched pair of aka relationships based ona pair of author IDs
    fn remove_aka_pair(&mut self, aid1: AuthorId, aid2: AuthorId) {
        let auth = self.authors.get(&aid1);
        if auth.is_none() || !auth.unwrap().akas.contains(&aid2) {
            return;
        }
        let auth = self.authors.get(&aid2);
        if auth.is_none() || !auth.unwrap().akas.contains(&aid1) {
            return;
        }
        let auth = self.authors.get_mut(&aid1).unwrap();
        let pos = vec_find(&auth.akas, aid2).unwrap();
        auth.akas.remove(pos);
        let auth = self.authors.get_mut(&aid2).unwrap();
        let pos = vec_find(&auth.akas, aid1).unwrap();
        auth.akas.remove(pos);
    }

    // remove an aka from a cluster represented by one member of the cluster
    pub fn remove_aka_cluster(&mut self, cluster: AuthorId, aka: AuthorId) {
        let removal = self.authors.get(&cluster);
        if removal.is_none() || !removal.unwrap().akas.contains(&cluster) {
            return;
        }
        let aka_copy = removal.unwrap().akas.clone();
        for id in aka_copy {
            self.remove_aka_pair(id, aka);
        }

        // at this point, the akas of the aka Author should be empty
    }

    // mere two aka clusters, represented by a member of each
    pub fn merge_aka_clusters(&mut self, c1: AuthorId, c2: AuthorId) {
        let auth = self.authors.get(&c1);
        if auth.is_none() || auth.unwrap().akas.contains(&c2) {
            return;
        }
        let c1_copy = auth.unwrap().akas.clone();
        let auth = self.authors.get(&c2);
        if auth.is_none() {
            return;
        }
        let c2_copy = auth.unwrap().akas.clone();
        // remember that the two aka copies will not have c1 or c2
        // first put together the two representatives
        self.add_aka_pair(c1, c2);
        for aka2 in c2_copy.clone() {
            self.add_aka_pair(c1, aka2);
        }
        // now fix all the other pairs
        for aka1 in c1_copy {
            self.add_aka_pair(aka1, c2);
            for aka2 in c2_copy.clone() {
                self.add_aka_pair(c1, aka2);
                self.add_aka_pair(aka1, aka2);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_library() {
        let mut lib = LibraryModel::new("TestName");
        let name = lib.name();
        assert_eq!(name, "TestName");
        lib.set_name("TestName2");
        let name = lib.name();
        assert_eq!(name, "TestName2");
        assert_eq!(lib.authors.len(), 2);
        let mut ai = lib.author_iter();
        let aid1 = ai.next();
        assert!(aid1.is_some());
        let aid1 = aid1.unwrap();
        let aid2 = ai.next();
        assert!(aid2.is_some());
        let aid2 = aid2.unwrap();
        let a1 = lib.get_author(aid1);
        assert!(a1.is_some());
        let a1 = a1.unwrap();
        let a2 = lib.get_author(aid2);
        assert!(a2.is_some());
        let a2 = a2.unwrap();
        assert_eq!(a1.akas.len(), 1);
        assert_eq!(a1.akas[0], *aid2);
        assert_eq!(a2.akas.len(), 1);
        assert_eq!(a2.akas[0], *aid1);
    }

    // should build a test for merge_aka_cluster and remove aka_cluster
    #[test]
    fn author_cluster_test() {
        let mut lib = LibraryModel::new("MergeTest");
        let names = ["n1", "n2", "3", "n4"];
        let alen = names.len() - 1;
        let authors = names
            .iter()
            .map(|name| AuthorModel::build(name, Vec::new(), Vec::new(), Vec::new()))
            .collect::<Vec<_>>();
        let aids = authors
            .iter()
            .map(|author| lib.add_author(author))
            .collect::<Vec<_>>();
        let mut aid_iter = aids.iter();
        let aid1 = aid_iter.next();
        let aid1 = aid1.unwrap();
        while let Some(aid) = aid_iter.next() {
            lib.merge_aka_clusters(*aid1, *aid);
        }
        let mut aid_iter = aids.iter();
        while let Some(aid) = aid_iter.next() {
            match &lib.get_author(aid) {
                None => panic!("Missing author after merge"),
                Some(author) => assert_eq!(author.akas.len(), alen),
            }
        }
    }
}
