use crate::author_model::AuthorModel;
use crate::library_model::{AuthorId, BookId};
use crate::library_window::author_list_gui::AuthorIdName;
use crate::library_window::{LibraryGuiState, Message};
use iced::widget::Column;
use iced::widget::{button, column, combo_box, row, rule, scrollable, text, text_input};
// use iced::widget::{pick_list, text_editr};
// use iced::Element;
use iced::Length;

// messages for the author list pane
#[derive(Debug, Clone)]
pub enum AuthorMessage {
    ChangeName(String),
    ApplyName,
    DeleteAka(AuthorId),
    SelectAka(AuthorIdName),
    AddAka,
    ToAKA(AuthorId, String),
    ToBook(BookId, String),
}

// state holder for the author gui panel
#[derive(Debug)]
pub struct AuthorGuiState {
    author: Option<AuthorId>,
    working_name: String,
    aka_choice: Option<AuthorIdName>,
    aka_combo: combo_box::State<AuthorIdName>,
}

impl Default for AuthorGuiState {
    fn default() -> Self {
        AuthorGuiState {
            author: None,
            working_name: "".to_string(),
            aka_choice: None,
            aka_combo: combo_box::State::new(Vec::new()),
        }
    }
}

impl Clone for AuthorGuiState {
    fn clone(&self) -> AuthorGuiState {
        AuthorGuiState {
            author: self.author,
            working_name: self.working_name.clone(),
            aka_choice: self.aka_choice.clone(),
            aka_combo: self.aka_combo.clone(),
        }
    }
}

impl AuthorGuiState {
    pub fn set_author(&mut self, aid: AuthorId, name: &str) {
        self.author = Some(aid);
        self.working_name = name.to_string();
    }

    pub fn clean(&mut self) {
        self.author = None;
        self.working_name = "".to_string();
        self.aka_choice = None;
        self.aka_combo = combo_box::State::new(Vec::new());
    }
}

fn mk_aka_combo(guistate: &mut LibraryGuiState, aid: AuthorId, akas: &[AuthorId]) {
    if let Some(library) = &guistate.library {
        let cakas = library
            .author_iter()
            .filter_map(|akaid| {
                if *akaid != aid && !akas.contains(akaid) {
                    Some(AuthorIdName {
                        id: *akaid,
                        name: library.get_author(akaid).unwrap().name.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();
        guistate.author_state.aka_combo = combo_box::State::new(cakas);
    }
}

// update handler for an individual author
pub fn author_update(
    guistate: &mut LibraryGuiState,
    message: AuthorMessage,
) -> iced::Task<Message> {
    match message {
        AuthorMessage::ChangeName(nstr) => {
            guistate.author_state.working_name = nstr.clone();
            guistate.mainpanel_dirty = true;
        }
        AuthorMessage::ApplyName => {
            if !guistate.author_state.working_name.is_empty()
                && let Some(lib) = &mut guistate.library
            {
                match &guistate.author_state.author {
                    Some(aid) => {
                        if let Some(auth) = &mut lib.mut_author(*aid) {
                            auth.set_name(&guistate.author_state.working_name);
                            guistate.mainpanel_dirty = false;
                            guistate.database_dirty = true;
                            let akas = auth.akas.clone();
                            mk_aka_combo(guistate, *aid, &akas);
                        }
                    }
                    None => {
                        let author = AuthorModel::build(
                            &guistate.author_state.working_name,
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                        );
                        let aid = lib.add_author(&author);
                        guistate.author_state.author = Some(aid);
                        guistate.mainpanel_dirty = false;
                        guistate.database_dirty = true;
                        mk_aka_combo(guistate, aid, &Vec::new());
                    }
                }
            }
        }
        AuthorMessage::DeleteAka(aid) => {
            if let Some(thisaid) = &guistate.author_state.author {
                guistate
                    .library
                    .as_mut()
                    .unwrap()
                    .remove_aka_cluster(*thisaid, aid);
                let akas = guistate
                    .library
                    .as_ref()
                    .unwrap()
                    .get_author(thisaid)
                    .unwrap()
                    .akas
                    .clone();
                mk_aka_combo(guistate, *thisaid, &akas);
            }
        }
        AuthorMessage::SelectAka(aidn) => {
            guistate.author_state.aka_choice = Some(aidn.clone());
            guistate.mainpanel_dirty = true;
        }
        AuthorMessage::AddAka => {
            if let Some(thisaid) = &guistate.author_state.author {
                if let Some(choice) = &guistate.author_state.aka_choice {
                    guistate
                        .library
                        .as_mut()
                        .unwrap()
                        .merge_aka_clusters(*thisaid, choice.id);
                    guistate.author_state.aka_choice = None;
                    guistate.mainpanel_dirty = false;
                    guistate.database_dirty = true;
                }
                let akas = guistate
                    .library
                    .as_ref()
                    .unwrap()
                    .get_author(thisaid)
                    .unwrap()
                    .akas
                    .clone();
                mk_aka_combo(guistate, *thisaid, &akas);
            }
        }
        AuthorMessage::ToAKA(aid, aname) => guistate.set_author(aid, &aname),
        AuthorMessage::ToBook(bid, _bname) => guistate.set_book(bid, false),
    }
    iced::Task::none()
}

// view function for an individual author
// handles both new and existing authors
pub fn author_view<'a>(guistate: &'a LibraryGuiState) -> Column<'a, Message> {
    if guistate.library.is_none() {
        return column![];
    }
    let lib = guistate.library.as_ref().unwrap();
    let opta = if let Some(author) = &guistate.author_state.author {
        lib.get_author(author)
    } else {
        None
    };

    let mut innercol = column![];
    let mut comborow = row![];
    let mut bookcol = column![];

    if let Some(auth) = opta {
        for akaid in auth
            .aka_iter()
            .filter(|akaid| lib.get_author(akaid).is_some())
        {
            innercol = innercol.push(row![
                button(text(lib.get_author(akaid).unwrap().name.clone()))
                    .on_press(Message::AuthorMessages(AuthorMessage::ToAKA(
                        *akaid,
                        auth.name.clone()
                    )))
                    .width(Length::FillPortion(9))
                    .style(iced::widget::button::text),
                button("del")
                    .on_press(Message::AuthorMessages(AuthorMessage::DeleteAka(*akaid)))
                    .style(iced::widget::button::danger)
                    .width(Length::FillPortion(1)),
            ]);
        }
        comborow = row![
            combo_box(
                &guistate.author_state.aka_combo,
                "Availalbe Authors",
                guistate.author_state.aka_choice.as_ref(),
                |aidn| Message::AuthorMessages(AuthorMessage::SelectAka(aidn)),
            ),
            button("Add").on_press_maybe(
                guistate
                    .author_state
                    .aka_choice
                    .is_some()
                    .then_some(Message::AuthorMessages(AuthorMessage::AddAka))
            ),
        ];
        bookcol = bookcol.push(text("Books:"));
        for bid in &auth.books {
            let book = lib.get_book(*bid);
            if let Some(abook) = book {
                bookcol = bookcol.push(
                    button(abook.title())
                        .on_press(Message::AuthorMessages(AuthorMessage::ToBook(
                            *bid,
                            abook.title().to_string(),
                        )))
                        .style(iced::widget::button::text),
                );
            }
        }
        bookcol = bookcol.push(text("Collections:"));
        for bid in &auth.appearences {
            let book = lib.get_book(*bid);
            if let Some(abook) = book {
                bookcol = bookcol.push(button(abook.title()).on_press(Message::AuthorMessages(
                    AuthorMessage::ToBook(*bid, abook.title().to_string()),
                )));
            }
        }
    }

    column![
        row![
            text("Author: "),
            text_input("", &guistate.author_state.working_name)
                .on_submit(Message::AuthorMessages(AuthorMessage::ApplyName))
                .on_input(|instr| Message::AuthorMessages(AuthorMessage::ChangeName(instr)))
                .width(Length::Fill),
        ],
        rule::horizontal(1),
        text("AKA s"),
        innercol,
        comborow,
        rule::horizontal(1),
        scrollable(bookcol),
    ]
}
