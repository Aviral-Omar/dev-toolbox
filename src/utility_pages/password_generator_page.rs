use {
    crate::{Message, app::AppModel, class::text_input_style, fl, utility_pages::UtilityPage},
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, clipboard,
            widget::{column, row},
        },
        widget::{self, Id, TextInput, text_input},
    },
    passwords::PasswordGenerator,
};

const PASSWORD_TEXT_ID: &str = "password-text";

#[derive(Debug, Clone)]
pub enum PasswordGeneratorMessage {
    SetLength(usize),
    SetNumbers(bool),
    SetLowercase(bool),
    SetUppercase(bool),
    SetSymbols(bool),
    SetExcludeSimilar(bool),
    GeneratePassword,
    SelectAllTextField(Id),
    CopyText(Id),
    ReadOnlyInput(Id),
}

pub(crate) struct PasswordGeneratorPage {
    length: usize,
    numbers: bool,
    lowercase: bool,
    uppercase: bool,
    symbols: bool,
    exclude_similar: bool,
    password_text: String,
}

impl Default for PasswordGeneratorPage {
    fn default() -> Self {
        PasswordGeneratorPage {
            length: 8,
            numbers: true,
            lowercase: true,
            uppercase: true,
            symbols: true,
            exclude_similar: true,
            password_text: String::default(),
        }
    }
}

fn clipboard_button(id: &str) -> Element<'_, Message> {
    widget::tooltip(
        widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
            Message::PasswordGeneratorMessage(PasswordGeneratorMessage::CopyText(Id::new(
                id.to_string(),
            ))),
        ),
        widget::text(fl!("copy")),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

impl UtilityPage for PasswordGeneratorPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let header = row![widget::text::title2(fl!("password-generator"))]
            .align_y(Alignment::End)
            .spacing(space_s);

        let options_header: Element<'_, Message> = widget::text::title4(fl!("options"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let length_option = widget::settings::item(
            fl!("password-generator", "length"),
            widget::spin_button(
                self.length.to_string(),
                fl!("password-generator", "length"),
                self.length,
                1,
                8,
                20,
                |value| {
                    Message::PasswordGeneratorMessage(PasswordGeneratorMessage::SetLength(value))
                },
            ),
        );

        let numbers_option = widget::settings::item(
            fl!("password-generator", "numbers"),
            widget::toggler(self.numbers).on_toggle(|value| {
                Message::PasswordGeneratorMessage(PasswordGeneratorMessage::SetNumbers(value))
            }),
        );

        let lowercase_option = widget::settings::item(
            fl!("password-generator", "lowercase"),
            widget::toggler(self.lowercase).on_toggle(|value| {
                Message::PasswordGeneratorMessage(PasswordGeneratorMessage::SetLowercase(value))
            }),
        );

        let uppercase_option = widget::settings::item(
            fl!("password-generator", "uppercase"),
            widget::toggler(self.uppercase).on_toggle(|value| {
                Message::PasswordGeneratorMessage(PasswordGeneratorMessage::SetUppercase(value))
            }),
        );

        let symbols_option = widget::settings::item(
            fl!("password-generator", "symbols"),
            widget::toggler(self.symbols).on_toggle(|value| {
                Message::PasswordGeneratorMessage(PasswordGeneratorMessage::SetSymbols(value))
            }),
        );

        let exclude_similar_option = widget::settings::item(
            fl!("password-generator", "exclude-similar"),
            widget::toggler(self.exclude_similar).on_toggle(|value| {
                Message::PasswordGeneratorMessage(PasswordGeneratorMessage::SetExcludeSimilar(
                    value,
                ))
            }),
        );

        let output_header: Element<'_, Message> = widget::text::title4(fl!("output"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let password_output: TextInput<'_, Message> = widget::text_input("", &self.password_text)
            .id(Id::new(PASSWORD_TEXT_ID))
            .style(text_input_style())
            .helper_text(fl!("password-generator", "password"))
            .editing(false)
            .trailing_icon(
                row![
                    widget::button::text(fl!("generate")).on_press(
                        Message::PasswordGeneratorMessage(
                            PasswordGeneratorMessage::GeneratePassword
                        )
                    ),
                    clipboard_button(PASSWORD_TEXT_ID),
                ]
                .into(),
            )
            .on_input(|_| {
                Message::PasswordGeneratorMessage(PasswordGeneratorMessage::ReadOnlyInput(Id::new(
                    PASSWORD_TEXT_ID,
                )))
            })
            .select_on_focus(true)
            .on_focus(Message::PasswordGeneratorMessage(
                PasswordGeneratorMessage::SelectAllTextField(Id::new(PASSWORD_TEXT_ID)),
            ));

        return column![
            header,
            options_header,
            length_option,
            numbers_option,
            lowercase_option,
            uppercase_option,
            symbols_option,
            exclude_similar_option,
            output_header,
            password_output
        ]
        .spacing(space_s)
        .height(Length::Fill)
        .into();
    }

    fn handle_message(
        &mut self,
        message: Message,
    ) -> Task<cosmic::Action<<AppModel as Application>::Message>> {
        match message {
            Message::PasswordGeneratorMessage(password_generator_message) => {
                match password_generator_message {
                    PasswordGeneratorMessage::SetLength(length) => {
                        self.length = length;
                    }
                    PasswordGeneratorMessage::SetNumbers(flag) => {
                        self.numbers = flag;
                    }
                    PasswordGeneratorMessage::SetLowercase(flag) => {
                        self.lowercase = flag;
                    }
                    PasswordGeneratorMessage::SetUppercase(flag) => {
                        self.uppercase = flag;
                    }
                    PasswordGeneratorMessage::SetSymbols(flag) => {
                        self.symbols = flag;
                    }
                    PasswordGeneratorMessage::SetExcludeSimilar(flag) => {
                        self.exclude_similar = flag;
                    }
                    PasswordGeneratorMessage::GeneratePassword => {
                        self.generate_password();
                    }
                    PasswordGeneratorMessage::SelectAllTextField(id) => {
                        return text_input::select_all(id);
                    }
                    PasswordGeneratorMessage::CopyText(id) => {
                        if id == Id::new(PASSWORD_TEXT_ID) {
                            return clipboard::write(self.password_text.clone());
                        }
                    }
                    PasswordGeneratorMessage::ReadOnlyInput(id) => {
                        return text_input::move_cursor_to_end(id);
                    }
                }
            }
            _ => {
                println!("Wrong message type");
            }
        };
        Task::none()
    }
}

impl PasswordGeneratorPage {
    fn generate_password(&mut self) {
        let pg = PasswordGenerator {
            length: self.length,
            numbers: self.numbers,
            lowercase_letters: self.lowercase,
            uppercase_letters: self.uppercase,
            symbols: self.symbols,
            spaces: false,
            exclude_similar_characters: self.exclude_similar,
            strict: true,
        };

        let password = pg.generate_one().unwrap();
        self.password_text = password;
    }
}
