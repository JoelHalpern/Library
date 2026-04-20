use crate::library_model::{AuthorId, BookId, LibraryModel};
use crate::library_window::author_gui::{
    AuthorGuiState, AuthorMessage, author_update, author_view,
};
use crate::library_window::author_list_gui::{
    AuthorListMessage, author_list_update, author_list_view,
};
use crate::library_window::book_gui::{BookGuiData, BookMessage, book_update, book_view};
use crate::library_window::book_list_gui::{
    BookListGuiData, BookListMessage, book_list_update, book_list_view, fix_filtered_books,
};
use iced::widget::{Column, Row};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length, Task};
use std::fs;
use std::fs::File;

pub mod author_gui;
pub mod author_list_gui;
pub mod book_gui;
pub mod book_list_gui;

pub fn library_gui_base() -> iced::Result {
    iced::application(
        LibraryGuiState::new,
        LibraryGuiState::update,
        LibraryGuiState::view,
    )
    .title("Rolodex")
    .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    NewLibrary,
    LoadLibrary,
    LoadFileSelected(rfd::FileHandle),
    FileRead(String),
    FileCancelled,
    SaveLibrary,
    SaveFileSelected(rfd::FileHandle),
    BooksPressed,
    AuthorsPressed,
    DiscardAll,
    AuthorListMessages(AuthorListMessage),
    AuthorMessages(AuthorMessage),
    BookListMessages(BookListMessage),
    BookMessages(BookMessage),
}

// the root of the iced gui state for this program
pub struct LibraryGuiState {
    library: Option<LibraryModel>,
    main_state: MainPanelGuiState,
    database_dirty: bool,
    mainpanel_dirty: bool,
    file_name: String,
    messageline: Result<String, String>,
    alt_control: bool,
    filter: String,
    author_state: author_gui::AuthorGuiState,
    book_state: book_gui::BookGuiData,
    book_list_state: book_list_gui::BookListGuiData,
}

impl Default for LibraryGuiState {
    fn default() -> Self {
        LibraryGuiState {
            library: None,
            main_state: MainPanelGuiState::default(),
            database_dirty: false,
            mainpanel_dirty: false,
            file_name: "".to_string(),
            messageline: Ok("".to_string()),
            alt_control: false,
            filter: "".to_string(),
            author_state: AuthorGuiState::default(),
            book_list_state: BookListGuiData::default(),
            book_state: BookGuiData::default(),
        }
    }
}

impl Clone for LibraryGuiState {
    fn clone(&self) -> LibraryGuiState {
        LibraryGuiState {
            library: self.library.clone(),
            main_state: self.main_state.clone(),
            database_dirty: self.database_dirty,
            mainpanel_dirty: self.mainpanel_dirty,
            file_name: self.file_name.clone(),
            messageline: self.messageline.clone(),
            alt_control: self.alt_control,
            filter: self.filter.clone(),
            author_state: self.author_state.clone(),
            book_list_state: self.book_list_state.clone(),
            book_state: self.book_state.clone(),
        }
    }
}

impl LibraryGuiState {
    fn new() -> Self {
        LibraryGuiState::default()
    }

    pub fn set_author(&mut self, aid: AuthorId, name: &str) {
        self.main_state.panel_state = MainPanelEnum::Author;
        self.author_state.set_author(aid, name);
    }

    pub fn set_book(&mut self, bid: BookId, edit: bool) {
        self.main_state.panel_state = MainPanelEnum::Book;
        if let Some(lib) = &self.library {
            if let Some(book) = &lib.get_book(bid) {
                self.book_state.set_content(lib, book, bid, edit)
            } else {
                // should be impossible, but define what to do anyway
                self.book_state.clean(lib, edit);
                self.book_state.fill_author_list(lib);
            }
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.messageline = Ok("".to_string());
        match message {
            Message::NewLibrary => {
                if !self.alt_control {
                    self.library = Some(LibraryModel::new("DummyName For now"));
                    self.main_state.panel_state = MainPanelEnum::Empty;
                    self.book_state.clean(self.library.as_ref().unwrap(), false);
                    self.book_list_state.clean();
                    self.author_state.clean();
                    self.database_dirty = false;
                    self.mainpanel_dirty = false;
                }
            }
            Message::FileCancelled => {
                self.alt_control = false;
                self.messageline = Ok("File Cancelled".to_string());
            }
            Message::LoadLibrary => {
                if !self.alt_control {
                    self.alt_control = true;
                    self.messageline = Ok("Selecting and Reading File".to_string());
                    return select_load_file();
                }
            }
            Message::LoadFileSelected(handle) => {
                self.file_name = handle.file_name().clone();
                return Task::perform(do_read_file(handle), Message::FileRead);
            }
            Message::FileRead(content) => {
                if content.is_empty() {
                    self.messageline = Ok("File Load cancelled.".to_string());
                    self.alt_control = false;
                } else {
                    let fr = serde_json::from_str(&content);
                    match fr {
                        Ok(lib) => {
                            self.library = Some(lib);
                            self.messageline = Ok("File Loaded".to_string());
                        }
                        Err(_) => {
                            self.messageline = Err("Error reading JSON file.".to_string());
                            self.library = None;
                        }
                    }
                    self.alt_control = false;
                }
            }
            Message::SaveLibrary => {
                if !self.alt_control {
                    self.alt_control = true;
                    self.messageline = Ok("Selecting and Saving Rolodex".to_string());
                    return select_save_file(self.file_name.clone());
                }
            }
            Message::SaveFileSelected(handle) => {
                let filep = handle.path();
                let mut bfp = filep.to_path_buf();
                bfp.push(".bak");
                let bfpp = bfp.as_path();
                if filep.exists() {
                    self.messageline = Ok("Renaming existing file".to_string());
                    if fs::rename(filep, bfpp).is_err() {
                        self.messageline = Err("Error Renaming file".to_string());
                    };
                }
                let file_result = File::create(filep);
                match file_result {
                    Ok(f) => {
                        self.library.as_mut().unwrap().set_name(&handle.file_name());
                        let _ = serde_json::to_writer(f, self.library.as_ref().unwrap());
                        self.database_dirty = false;
                        self.messageline = Ok("Saved rolodex".to_string());
                    }
                    Err(_) => self.messageline = Err("Error creating save file.".to_string()),
                }
                self.alt_control = false;
            }
            Message::BooksPressed => {
                self.book_list_state.clean();
                fix_filtered_books(self);
                self.main_state.panel_state = MainPanelEnum::BookList;
            }
            Message::AuthorsPressed => self.main_state.panel_state = MainPanelEnum::AuthorList,
            Message::DiscardAll => {
                if !self.alt_control {
                    self.library = Some(LibraryModel::new("DmmyName for now"));
                    self.main_state.panel_state = MainPanelEnum::Empty;
                    self.author_state.clean();
                    self.book_state.clean(self.library.as_ref().unwrap(), false);
                    self.database_dirty = false;
                    self.mainpanel_dirty = false;
                }
            }
            Message::AuthorListMessages(alm) => return author_list_update(self, alm),
            Message::AuthorMessages(am) => return author_update(self, am),
            Message::BookListMessages(blm) => return book_list_update(self, blm),
            Message::BookMessages(bm) => return book_update(self, bm),
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let msgtext = match &self.messageline {
            Ok(line) => text(line.clone()),
            Err(line) => text(line.clone()).style(text::warning),
        };
        container(column![
            self.top_panel().width(Length::Fill),
            msgtext,
            scrollable(self.library_main_panel().width(Length::Fill)).spacing(2),
        ])
        .into()
    }

    // display the top row with the rolodex name and buttons
    fn top_panel(&self) -> Row<'_, Message> {
        let mut lname = "";
        let mut b1 = button("New");
        let mut b2 = button("Load");
        let mut b3 = button("Save");
        let mut b4 = button("Books");
        let mut b5 = button("Authors");
        let mut b6 = button("Discard All");

        match &self.library {
            None => {
                b1 = b1.on_press(Message::NewLibrary);
                b2 = b2.on_press(Message::LoadLibrary);
            }

            Some(lib) => {
                lname = lib.name();
                if !self.alt_control && !self.database_dirty {
                    b1 = b1.on_press(Message::NewLibrary);
                }
                if !self.alt_control && !self.database_dirty {
                    b2 = b2.on_press(Message::LoadLibrary)
                };
                if !self.alt_control && !self.mainpanel_dirty && self.database_dirty {
                    b3 = b3.on_press(Message::SaveLibrary)
                };
                if !self.alt_control && !self.mainpanel_dirty {
                    b4 = b4.on_press(Message::BooksPressed)
                };
                if !self.alt_control && !self.mainpanel_dirty {
                    b5 = b5.on_press(Message::AuthorsPressed)
                }
                if !self.alt_control {
                    b6 = b6.on_press(Message::DiscardAll);
                }
            }
        }

        row![text("Library"), text(lname), b1, b2, b3, b4, b5, b6,].spacing(4)
    }

    // dispatch to the various possible main panel display routines
    fn library_main_panel(&self) -> Column<'_, Message> {
        if self.alt_control {
            // display nothing if async file operations are running
            column![]
        } else {
            match self.main_state.panel_state {
                MainPanelEnum::Empty => column![],
                MainPanelEnum::BookList => book_list_view(self),
                MainPanelEnum::AuthorList => author_list_view(self),
                MainPanelEnum::Book => book_view(self),
                MainPanelEnum::Author => author_view(self),
            }
        }
    }
}

#[derive(Debug, Clone)]
enum MainPanelEnum {
    Empty,
    BookList,
    AuthorList,
    Book,
    Author,
}

#[derive(Debug)]
struct MainPanelGuiState {
    panel_state: MainPanelEnum,
}

impl Default for MainPanelGuiState {
    fn default() -> Self {
        MainPanelGuiState {
            panel_state: MainPanelEnum::Empty,
        }
    }
}

impl Clone for MainPanelGuiState {
    fn clone(&self) -> MainPanelGuiState {
        MainPanelGuiState {
            panel_state: match &self.panel_state {
                MainPanelEnum::Empty => MainPanelEnum::Empty,
                MainPanelEnum::BookList => MainPanelEnum::BookList,
                MainPanelEnum::AuthorList => MainPanelEnum::AuthorList,
                MainPanelEnum::Book => MainPanelEnum::Book,
                MainPanelEnum::Author => MainPanelEnum::Author,
            },
        }
    }
}

// invoke an rfd file selection async dialog for a file to load,
// and send back a suitable message to the iced dispatcher
fn select_load_file() -> Task<Message> {
    Task::future(
        rfd::AsyncFileDialog::new()
            .set_title("Open a Rolodex")
            .add_filter("json", &["json"])
            .pick_file(),
    )
    .then(|handle| match handle {
        // The user has cancelled the operation, so we return a "Cancelled" message.
        None => Task::done(Message::FileCancelled),
        Some(file_handle) => Task::done(Message::LoadFileSelected(file_handle)),
    })
}

// invoke an rfd file selector async dialog for a faile to save into,
// and send back a suitable message to the iced dispatcher
fn select_save_file(fname: String) -> Task<Message> {
    Task::future(rfd::AsyncFileDialog::new().set_file_name(fname).save_file()).then(|handle| {
        match handle {
            Some(handle) => Task::done(Message::SaveFileSelected(handle)),
            None => Task::done(Message::FileCancelled),
        }
    })
}

// perform the asynchronous read of the file.
async fn do_read_file(handle: rfd::FileHandle) -> String {
    let content = handle.read().await;
    String::from_utf8(content).expect("Error converting dataa from file.")
}
