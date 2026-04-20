use crate::library_model::{AuthorId, LibraryModel};
use crate::library_window::{LibraryGuiState, MainPanelEnum, Message};
use iced::Length;
use iced::widget::Column;
use iced::widget::{button, column, row, scrollable, text, text_input};

// messages for the author list pane
#[derive(Debug, Clone)]
pub enum AuthorListMessage {
    DisplayAuthor(AuthorId, String),
    AddAuthor,
    DelAuthor(AuthorId),
    Search(String),
}

// update handler for the author list screen
pub fn author_list_update(
    guistate: &mut LibraryGuiState,
    message: AuthorListMessage,
) -> iced::Task<Message> {
    match message {
        AuthorListMessage::DisplayAuthor(aid, name) => guistate.set_author(aid, &name),
        AuthorListMessage::AddAuthor => {
            guistate.main_state.panel_state = MainPanelEnum::Author;
            guistate.author_state.clean();
        }
        AuthorListMessage::DelAuthor(aid) => {
            guistate.library.as_mut().unwrap().delete_author(aid);
            guistate.database_dirty = true;
        }
        AuthorListMessage::Search(instr) => guistate.filter = instr.trim().to_string(),
    }
    iced::Task::none()
}

// view handler for the author lsit screen
pub fn author_list_view<'a>(guistate: &'a LibraryGuiState) -> Column<'a, Message> {
    let lib = guistate.library.as_ref().unwrap();
    let list = mk_author_list(lib, &guistate.filter);

    column![
        row![
            text("Author Filter: ").width(Length::FillPortion(2)),
            text_input("", &guistate.filter)
                .on_input(|instr| Message::AuthorListMessages(AuthorListMessage::Search(instr)))
                .width(Length::FillPortion(5)),
            text("    ").width(Length::FillPortion(2)),
            button("Add")
                .on_press(Message::AuthorListMessages(AuthorListMessage::AddAuthor))
                .width(Length::FillPortion(1)),
        ]
        .spacing(2),
        scrollable(
            column(list.into_iter().map(|aidn| {
                row![
                    button(text(aidn.name.clone()))
                        .on_press(Message::AuthorListMessages(
                            AuthorListMessage::DisplayAuthor(aidn.id, aidn.name.to_string())
                        ))
                        .width(Length::FillPortion(9))
                        .style(iced::widget::button::text),
                    button("del")
                        .on_press_maybe(lib.check_auth_noref(aidn.id).then_some(
                            Message::AuthorListMessages(AuthorListMessage::DelAuthor(aidn.id,))
                        ))
                        .style(iced::widget::button::danger)
                        .width(Length::FillPortion(1)),
                ]
                .spacing(1)
                .into()
            }))
            .spacing(1)
        ),
    ]
}

// Author ID, Name pair
#[derive(Debug, Clone)]
pub struct AuthorIdName {
    pub id: AuthorId,
    pub name: String,
}

// a fromater for the AuthorIDName that formats just the name part so
// that a combo_box can use it for selection
impl std::fmt::Display for AuthorIdName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

pub fn mk_author_list(lib: &LibraryModel, filter: &str) -> Vec<AuthorIdName> {
    let mut list = Vec::<AuthorIdName>::new();

    for id in lib.author_iter() {
        if let Some(auth) = lib.get_author(id)
            && auth.name.contains(filter)
        {
            list.push(AuthorIdName {
                id: *id,
                name: auth.name.clone(),
            });
        }
    }
    list.sort_by(|aidn1, aidn2| aidn1.name.cmp(&aidn2.name));
    list
}
