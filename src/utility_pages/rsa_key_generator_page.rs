use {
    crate::{
        Message, app::AppModel, class::text_editor_class, fl, i18n::LANGUAGE_LOADER,
        utility_pages::UtilityPage,
    },
    cosmic::{
        self, Application, Element, Task,
        iced::{
            self, Alignment, Length, Padding, clipboard,
            keyboard::{Key, key},
            widget::{column, row},
        },
        widget::{
            self, Id,
            text_editor::{self, Binding, TextEditor},
        },
    },
    rand,
    rsa::{
        RsaPrivateKey,
        pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    },
    std::sync::Arc,
};

const BITS: [&str; 3] = ["1024", "2048", "4196"];
const PRIVATE_KEY_EDITOR_ID: &str = "private-key-editor";
const PUBLIC_KEY_EDITOR_ID: &str = "public-key-editor";

#[derive(Debug, Clone)]
pub enum RSAKeyGeneratorMessage {
    PrivateKeyEditorAction(text_editor::Action),
    PublicKeyEditorAction(text_editor::Action),
    ExtractPublicKey,
    GenerateKeyPair,
    GenerateKeyPairResult(Result<RsaPrivateKey, String>),
    BitsChanged(usize),
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    NoOp,
}

pub(crate) struct RSAKeyGeneratorPage {
    public_key_content: text_editor::Content,
    private_key_content: text_editor::Content,
    bits: usize,
    status: String,
}

impl Default for RSAKeyGeneratorPage {
    fn default() -> Self {
        Self {
            public_key_content: text_editor::Content::default(),
            private_key_content: text_editor::Content::default(),
            bits: usize::default(),
            status: "ok".to_string(),
        }
    }
}

fn replace_text_in_field(content: &mut text_editor::Content, text: String) {
    content.perform(text_editor::Action::SelectAll);
    content.perform(text_editor::Action::Edit(text_editor::Edit::Paste(
        Arc::new(text),
    )));
}

async fn generate_key_pair(bits: usize) -> Result<RsaPrivateKey, String> {
    let bits = usize::pow(2, 10 + bits as u32);
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, bits);
    return match private_key {
        Ok(private_key) => Ok(private_key),
        Err(_) => Err("keygen-failed".to_string()),
    };
}

impl UtilityPage for RSAKeyGeneratorPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("rsa-key-generator")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let options_header: Element<'_, Message> = row![
            widget::text::title4(fl!("options"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
        ]
        .into();

        let bits_option: Element<'_, Message> = widget::settings::item(
            fl!("rsa-key-generator", "bits"),
            widget::dropdown(BITS.to_vec(), Some(self.bits), |selection| {
                Message::RSAKeyGeneratorMessage(RSAKeyGeneratorMessage::BitsChanged(selection))
            }),
        )
        .into();

        let private_key_header: Element<'_, Message> = row![
            widget::text::heading(fl!("rsa-key-generator", "private-key"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::text(fl!("extract")).on_press(Message::RSAKeyGeneratorMessage(
                    RSAKeyGeneratorMessage::ExtractPublicKey
                )),
                widget::text(fl!("rsa-key-generator", "extract-public-key")),
                widget::tooltip::Position::Bottom,
            ),
            widget::tooltip(
                widget::button::text(fl!("generate")).on_press(Message::RSAKeyGeneratorMessage(
                    RSAKeyGeneratorMessage::GenerateKeyPair
                )),
                widget::text(fl!("rsa-key-generator", "generate-key-pair")),
                widget::tooltip::Position::Bottom,
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::RSAKeyGeneratorMessage(RSAKeyGeneratorMessage::CopyText(Id::new(
                        PRIVATE_KEY_EDITOR_ID
                    )),),
                ),
                widget::text(fl!("copy")),
                widget::tooltip::Position::Bottom,
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::RSAKeyGeneratorMessage(RSAKeyGeneratorMessage::PasteText(Id::new(
                        PRIVATE_KEY_EDITOR_ID
                    )),)
                ),
                widget::text(fl!("paste")),
                widget::tooltip::Position::Bottom,
            ),
        ]
        .into();

        let private_key_editor: Element<'_, Message> = TextEditor::new(&self.private_key_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced::core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::RSAKeyGeneratorMessage(RSAKeyGeneratorMessage::PrivateKeyEditorAction(
                    action,
                ))
            })
            .key_binding(|key_press| {
                if key_press.key == Key::Named(key::Named::Tab)
                    && matches!(key_press.status, text_editor::Status::Focused { .. })
                {
                    return Some(Binding::Insert('\t'));
                }
                return Binding::from_key_press(key_press);
            })
            .into();

        let public_key_header: Element<'_, Message> = row![
            widget::text::heading(fl!("rsa-key-generator", "public-key"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::RSAKeyGeneratorMessage(RSAKeyGeneratorMessage::CopyText(Id::new(
                        PUBLIC_KEY_EDITOR_ID
                    )),),
                ),
                widget::text(fl!("copy")),
                widget::tooltip::Position::Bottom,
            ),
        ]
        .align_y(Alignment::Center)
        .into();

        let public_key_editor: Element<'_, Message> = TextEditor::new(&self.public_key_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced::core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::RSAKeyGeneratorMessage(RSAKeyGeneratorMessage::PublicKeyEditorAction(
                    action,
                ))
            })
            .into();

        let status_row: Element<'_, Message> = row![
            widget::text::heading(fl!("status")),
            widget::space().width(8),
            widget::text::body(LANGUAGE_LOADER.get_attr("rsa-key-generator", self.status.as_str())),
        ]
        .into();

        column![
            header,
            options_header,
            bits_option,
            private_key_header,
            private_key_editor,
            public_key_header,
            public_key_editor,
            column![status_row]
                .width(Length::Fill)
                .align_x(Alignment::Center),
        ]
        .spacing(space_s)
        .height(Length::Fill)
        .into()
    }

    fn handle_message(
        &mut self,
        message: Message,
    ) -> Task<cosmic::Action<<AppModel as Application>::Message>> {
        match message {
            Message::RSAKeyGeneratorMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    RSAKeyGeneratorMessage::PrivateKeyEditorAction(action) => {
                        self.private_key_content.perform(action);
                    }
                    RSAKeyGeneratorMessage::PublicKeyEditorAction(action) => {
                        if !action.is_edit() {
                            self.public_key_content.perform(action);
                        }
                    }
                    RSAKeyGeneratorMessage::BitsChanged(selection) => {
                        self.bits = selection;
                    }
                    RSAKeyGeneratorMessage::ExtractPublicKey => {
                        self.extract_public_key();
                    }
                    RSAKeyGeneratorMessage::GenerateKeyPair => {
                        let bits = self.bits;
                        self.status = "generating".to_string();
                        return Task::future(generate_key_pair(bits)).map(move |result| {
                            cosmic::Action::App(Message::RSAKeyGeneratorMessage(
                                RSAKeyGeneratorMessage::GenerateKeyPairResult(result),
                            ))
                        });
                    }
                    RSAKeyGeneratorMessage::GenerateKeyPairResult(result) => {
                        self.handle_generate_result(result);
                    }
                    RSAKeyGeneratorMessage::CopyText(id) => {
                        let mut to_copy: String = String::new();
                        if id == Id::new(PRIVATE_KEY_EDITOR_ID) {
                            to_copy = self.private_key_content.text();
                        } else if id == Id::new(PUBLIC_KEY_EDITOR_ID) {
                            to_copy = self.public_key_content.text();
                        }
                        return clipboard::write(to_copy);
                    }
                    RSAKeyGeneratorMessage::PasteText(id) => {
                        return clipboard::read().map(move |optional_data| match optional_data {
                            Some(data) => cosmic::Action::App(Message::RSAKeyGeneratorMessage(
                                RSAKeyGeneratorMessage::ReplaceText(id.clone(), data),
                            )),
                            None => cosmic::Action::App(Message::RSAKeyGeneratorMessage(
                                RSAKeyGeneratorMessage::NoOp,
                            )),
                        });
                    }
                    RSAKeyGeneratorMessage::ReplaceText(id, text) => {
                        if id == Id::new(PUBLIC_KEY_EDITOR_ID) {
                            replace_text_in_field(&mut self.public_key_content, text);
                        } else if id == Id::new(PRIVATE_KEY_EDITOR_ID) {
                            replace_text_in_field(&mut self.private_key_content, text);
                        }
                    }
                    RSAKeyGeneratorMessage::NoOp => {}
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl RSAKeyGeneratorPage {
    fn extract_public_key(&mut self) {
        let input = self.private_key_content.text();
        let private_key = RsaPrivateKey::from_pkcs8_pem(input.as_str());
        match private_key {
            Ok(private_key) => {
                let public_key_str = private_key
                    .to_public_key()
                    .to_public_key_pem(LineEnding::default())
                    .unwrap();
                self.status = "ok".to_string();
                self.public_key_content
                    .perform(text_editor::Action::SelectAll);
                self.public_key_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.public_key_content.perform(text_editor::Action::Edit(
                    text_editor::Edit::Paste(Arc::new(public_key_str)),
                ));
            }
            Err(_) => self.status = "invalid-private-key".to_string(),
        }
    }

    fn handle_generate_result(&mut self, private_key_result: Result<RsaPrivateKey, String>) {
        match private_key_result {
            Ok(private_key) => {
                self.private_key_content
                    .perform(text_editor::Action::SelectAll);
                self.private_key_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.private_key_content.perform(text_editor::Action::Edit(
                    text_editor::Edit::Paste(Arc::new(
                        private_key
                            .to_pkcs8_pem(LineEnding::LF)
                            .unwrap()
                            .to_string(),
                    )),
                ));
                let public_key = private_key.to_public_key();
                self.public_key_content
                    .perform(text_editor::Action::SelectAll);
                self.public_key_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.public_key_content.perform(text_editor::Action::Edit(
                    text_editor::Edit::Paste(Arc::new(
                        public_key.to_public_key_pem(LineEnding::LF).unwrap(),
                    )),
                ));
                self.status = "ok".to_string();
            }
            Err(err) => self.status = err,
        }
    }
}
