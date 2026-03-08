use {
    crate::{
        Message,
        app::AppModel,
        class::{text_editor_class, text_input_style},
        fl,
        i18n::LANGUAGE_LOADER,
        utility_pages::UtilityPage,
    },
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, Padding, clipboard,
            keyboard::{Key, key},
            widget::{column, row},
        },
        iced_core, iced_widget,
        widget::{
            self, Id,
            text_editor::{self, Binding, TextEditor},
        },
    },
    jsonwebtoken::{self, Algorithm},
    std::sync::Arc,
};

const TOKEN_EDITOR_ID: &str = "token-editor";
const HEADER_EDITOR_ID: &str = "header-editor";
const CLAIMS_EDITOR_ID: &str = "claims-editor";
const KEY_ENCODING: [&str; 2] = ["utf-8", "base64"];
const SYMMETRIC_KEY_TEXT_ID: &str = "symmetric-key-text";
const PUBLIC_KEY_EDITOR_ID: &str = "public-key-editor";
const PRIVATE_KEY_EDITOR_ID: &str = "private-key-editor";

#[derive(Debug, Clone)]
pub enum JwtDebuggerMessage {
    TokenEditorAction(text_editor::Action),
    HeaderEditorAction(text_editor::Action),
    ClaimsEditorAction(text_editor::Action),
    PublicKeyEditorAction(text_editor::Action),
    PrivateKeyEditorAction(text_editor::Action),
    KeyEncodingChanged(usize),
    SymmetricKeyChanged(String),
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    NoOp,
}

enum Operation {
    Encode,
    Decode,
}

pub(crate) struct JwtDebuggerPage {
    token_content: text_editor::Content,
    header_content: text_editor::Content,
    claims_content: text_editor::Content,
    public_key_content: text_editor::Content,
    private_key_content: text_editor::Content,
    algorithm: Algorithm,
    key_encoding: usize,
    symmetric_key: String,
    status: String,
    last_operation: Operation,
}

#[derive(PartialEq)]
enum AlgorithmType {
    Symmetric,
    Asymmetric,
}

impl Default for JwtDebuggerPage {
    fn default() -> Self {
        Self {
            token_content: text_editor::Content::default(),
            header_content: text_editor::Content::default(),
            claims_content: text_editor::Content::default(),
            public_key_content: text_editor::Content::default(),
            private_key_content: text_editor::Content::default(),
            algorithm: Algorithm::default(),
            key_encoding: usize::default(),
            symmetric_key: String::default(),
            status: "ok".to_string(),
            last_operation: Operation::Decode,
        }
    }
}

// fn algorithm_from_str(algorithm: &str) -> Option<Algorithm> {
//     match algorithm {
//         "HS256" => Some(Algorithm::HS256),
//         "HS384" => Some(Algorithm::HS384),
//         "HS512" => Some(Algorithm::HS512),
//         "RS256" => Some(Algorithm::RS256),
//         "RS384" => Some(Algorithm::RS384),
//         "RS512" => Some(Algorithm::RS512),
//         "ES256" => Some(Algorithm::ES256),
//         "ES384" => Some(Algorithm::ES384),
//         "PS256" => Some(Algorithm::PS256),
//         "PS384" => Some(Algorithm::PS384),
//         "PS512" => Some(Algorithm::PS512),
//         "EdDSA" => Some(Algorithm::EdDSA),
//         _ => None,
//     }
// }

fn algorithm_type(algorithm: Algorithm) -> AlgorithmType {
    match algorithm {
        Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => AlgorithmType::Symmetric,
        _ => AlgorithmType::Asymmetric,
    }
}

fn replace_text_in_field(content: &mut text_editor::Content, text: String) {
    content.perform(text_editor::Action::SelectAll);
    content.perform(text_editor::Action::Edit(text_editor::Edit::Paste(
        Arc::new(text),
    )));
}

impl UtilityPage for JwtDebuggerPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("jwt-debugger")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let options_header: Element<'_, Message> = row![
            widget::text::title4(fl!("options"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
        ]
        .into();

        let token_header: Element<'_, Message> = row![
            widget::text::title4(fl!("jwt-debugger", "token"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::CopyText(Id::new(
                        TOKEN_EDITOR_ID
                    )),),
                ),
                widget::text(fl!("copy")),
                iced_widget::tooltip::Position::Bottom,
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::PasteText(Id::new(
                        TOKEN_EDITOR_ID
                    )),)
                ),
                widget::text(fl!("paste")),
                iced_widget::tooltip::Position::Bottom,
            ),
        ]
        .into();

        let token_editor: Element<'_, Message> = TextEditor::new(&self.token_content)
            .padding(Padding::new(12.0))
            .min_height(200)
            .height(Length::FillPortion(2))
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced_core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::JwtDebuggerMessage(JwtDebuggerMessage::TokenEditorAction(action))
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

        let header_header: Element<'_, Message> = row![
            widget::text::title4(fl!("jwt-debugger", "header"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::CopyText(Id::new(
                        HEADER_EDITOR_ID
                    )),),
                ),
                widget::text(fl!("copy")),
                iced_widget::tooltip::Position::Bottom,
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::PasteText(Id::new(
                        HEADER_EDITOR_ID
                    )),)
                ),
                widget::text(fl!("paste")),
                iced_widget::tooltip::Position::Bottom,
            ),
        ]
        .align_y(Alignment::Center)
        .into();

        let header_editor: Element<'_, Message> = TextEditor::new(&self.header_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced_core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::JwtDebuggerMessage(JwtDebuggerMessage::HeaderEditorAction(action))
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

        let claims_header: Element<'_, Message> = row![
            widget::text::title4(fl!("jwt-debugger", "claims"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::CopyText(Id::new(
                        CLAIMS_EDITOR_ID
                    )),),
                ),
                widget::text(fl!("copy")),
                iced_widget::tooltip::Position::Bottom,
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::PasteText(Id::new(
                        CLAIMS_EDITOR_ID
                    )),)
                ),
                widget::text(fl!("paste")),
                iced_widget::tooltip::Position::Bottom,
            ),
        ]
        .align_y(Alignment::Center)
        .into();

        let claims_editor: Element<'_, Message> = TextEditor::new(&self.claims_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced_core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::JwtDebuggerMessage(JwtDebuggerMessage::ClaimsEditorAction(action))
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

        let status_row: Element<'_, Message> = row![
            widget::text::heading(fl!("status")),
            widget::space().width(8),
            widget::text::body(LANGUAGE_LOADER.get_attr("jwt-debugger", self.status.as_str())),
        ]
        .into();

        let mut column = widget::column::with_capacity(10)
            .spacing(space_s)
            .height(Length::Fill);
        column = column.push(header).push(options_header);

        if algorithm_type(self.algorithm) == AlgorithmType::Symmetric {
            let key_encoding_option: Element<'_, Message> = widget::settings::item(
                fl!("jwt-debugger", "key-encoding"),
                widget::dropdown(
                    KEY_ENCODING
                        .map(|key_encoding| LANGUAGE_LOADER.get_attr("jwt-debugger", key_encoding))
                        .to_vec(),
                    Some(self.key_encoding),
                    |selection| {
                        Message::JwtDebuggerMessage(JwtDebuggerMessage::KeyEncodingChanged(
                            selection,
                        ))
                    },
                ),
            )
            .into();
            let symmetric_key_text = widget::text_input::text_input("", &self.symmetric_key)
                .id(Id::new(SYMMETRIC_KEY_TEXT_ID))
                .style(text_input_style())
                .helper_text(fl!("jwt-debugger", "symmetric-key"))
                .trailing_icon(
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name("edit-paste-symbolic"))
                            .on_press(Message::JwtDebuggerMessage(JwtDebuggerMessage::PasteText(
                                Id::new(SYMMETRIC_KEY_TEXT_ID),
                            ))),
                        widget::text(fl!("paste")),
                        iced_widget::tooltip::Position::Bottom,
                    )
                    .into(),
                )
                .on_input(|input| {
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::SymmetricKeyChanged(input))
                });
            column = column.push(key_encoding_option).push(symmetric_key_text);
        } else {
            let public_key_header: Element<'_, Message> =
                row![
                    widget::text::heading(fl!("jwt-debugger", "public-key"))
                        .width(Length::Fill)
                        .align_x(Alignment::Start),
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                            .on_press(Message::JwtDebuggerMessage(JwtDebuggerMessage::CopyText(
                                Id::new(PUBLIC_KEY_EDITOR_ID)
                            ),),),
                        widget::text(fl!("copy")),
                        iced_widget::tooltip::Position::Bottom,
                    ),
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name("edit-paste-symbolic"))
                            .on_press(Message::JwtDebuggerMessage(JwtDebuggerMessage::PasteText(
                                Id::new(PUBLIC_KEY_EDITOR_ID)
                            ),)),
                        widget::text(fl!("paste")),
                        iced_widget::tooltip::Position::Bottom,
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
                .wrapping(iced_core::text::Wrapping::WordOrGlyph)
                .on_action(|action| {
                    Message::JwtDebuggerMessage(JwtDebuggerMessage::PublicKeyEditorAction(action))
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

            let private_key_header: Element<'_, Message> =
                row![
                    widget::text::heading(fl!("jwt-debugger", "private-key"))
                        .width(Length::Fill)
                        .align_x(Alignment::Start),
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                            .on_press(Message::JwtDebuggerMessage(JwtDebuggerMessage::CopyText(
                                Id::new(PRIVATE_KEY_EDITOR_ID)
                            ),),),
                        widget::text(fl!("copy")),
                        iced_widget::tooltip::Position::Bottom,
                    ),
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name("edit-paste-symbolic"))
                            .on_press(Message::JwtDebuggerMessage(JwtDebuggerMessage::PasteText(
                                Id::new(PRIVATE_KEY_EDITOR_ID)
                            ),)),
                        widget::text(fl!("paste")),
                        iced_widget::tooltip::Position::Bottom,
                    ),
                ]
                .align_y(Alignment::Center)
                .into();

            let private_key_editor: Element<'_, Message> =
                TextEditor::new(&self.private_key_content)
                    .padding(Padding::new(12.0))
                    .height(Length::Fill)
                    .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                        text_editor_class,
                    )))
                    .wrapping(iced_core::text::Wrapping::WordOrGlyph)
                    .on_action(|action| {
                        Message::JwtDebuggerMessage(JwtDebuggerMessage::PrivateKeyEditorAction(
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

            column = column.push(
                row![
                    column![public_key_header, public_key_editor].spacing(space_s),
                    column![private_key_header, private_key_editor].spacing(space_s)
                ]
                .height(Length::FillPortion(1))
                .spacing(space_s),
            );
        }

        column = column
            .push(token_header)
            .push(token_editor)
            .push(
                row![
                    column![header_header, header_editor].spacing(space_s),
                    column![claims_header, claims_editor].spacing(space_s)
                ]
                .height(Length::FillPortion(3))
                .spacing(space_s),
            )
            .push(
                column![status_row]
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            );
        column.into()
    }

    fn handle_message(
        &mut self,
        message: Message,
    ) -> Task<cosmic::Action<<AppModel as Application>::Message>> {
        match message {
            Message::JwtDebuggerMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    JwtDebuggerMessage::TokenEditorAction(action) => {
                        let is_edit = matches!(action, text_editor::Action::Edit(_));
                        self.token_content.perform(action);
                        if is_edit {
                            self.decode_token();
                        }
                    }
                    JwtDebuggerMessage::HeaderEditorAction(action) => {
                        let is_edit = matches!(action, text_editor::Action::Edit(_));
                        self.header_content.perform(action);
                        if is_edit {
                            self.encode_token();
                        }
                    }
                    JwtDebuggerMessage::ClaimsEditorAction(action) => {
                        let is_edit = matches!(action, text_editor::Action::Edit(_));
                        self.claims_content.perform(action);
                        if is_edit {
                            self.encode_token();
                        }
                    }
                    JwtDebuggerMessage::PublicKeyEditorAction(action) => {
                        let is_edit = matches!(action, text_editor::Action::Edit(_));
                        self.public_key_content.perform(action);
                        if is_edit {
                            self.decode_token();
                        }
                    }
                    JwtDebuggerMessage::PrivateKeyEditorAction(action) => {
                        let is_edit = matches!(action, text_editor::Action::Edit(_));
                        self.private_key_content.perform(action);
                        if is_edit {
                            self.encode_token();
                        }
                    }
                    JwtDebuggerMessage::KeyEncodingChanged(selection) => {
                        self.key_encoding = selection;
                        self.perform_last_operation();
                    }
                    JwtDebuggerMessage::SymmetricKeyChanged(input) => {
                        self.symmetric_key = input;
                    }
                    JwtDebuggerMessage::CopyText(id) => {
                        let mut to_copy: String = String::new();
                        if id == Id::new(TOKEN_EDITOR_ID) {
                            to_copy = self.token_content.text();
                        } else if id == Id::new(HEADER_EDITOR_ID) {
                            to_copy = self.header_content.text();
                        } else if id == Id::new(CLAIMS_EDITOR_ID) {
                            to_copy = self.claims_content.text();
                        } else if id == Id::new(PUBLIC_KEY_EDITOR_ID) {
                            to_copy = self.public_key_content.text();
                        } else if id == Id::new(PRIVATE_KEY_EDITOR_ID) {
                            to_copy = self.private_key_content.text();
                        }
                        return clipboard::write(to_copy);
                    }
                    JwtDebuggerMessage::PasteText(id) => {
                        return clipboard::read().map(move |optional_data| match optional_data {
                            Some(data) => cosmic::Action::App(Message::JwtDebuggerMessage(
                                JwtDebuggerMessage::ReplaceText(id.clone(), data),
                            )),
                            None => cosmic::Action::App(Message::JwtDebuggerMessage(
                                JwtDebuggerMessage::NoOp,
                            )),
                        });
                    }
                    JwtDebuggerMessage::ReplaceText(id, text) => {
                        if id == Id::new(TOKEN_EDITOR_ID) {
                            replace_text_in_field(&mut self.token_content, text);
                            self.decode_token();
                        } else if id == Id::new(HEADER_EDITOR_ID) {
                            replace_text_in_field(&mut self.header_content, text);
                            self.encode_token();
                        } else if id == Id::new(CLAIMS_EDITOR_ID) {
                            replace_text_in_field(&mut self.claims_content, text);
                            self.encode_token();
                        } else if id == Id::new(PUBLIC_KEY_EDITOR_ID) {
                            replace_text_in_field(&mut self.public_key_content, text);
                            self.decode_token();
                        } else if id == Id::new(PRIVATE_KEY_EDITOR_ID) {
                            replace_text_in_field(&mut self.private_key_content, text);
                            self.encode_token();
                        } else if id == Id::new(SYMMETRIC_KEY_TEXT_ID) {
                            self.symmetric_key = text;
                            self.perform_last_operation();
                        }
                    }
                    JwtDebuggerMessage::NoOp => {}
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl JwtDebuggerPage {
    fn decode_token(&mut self) {
        self.last_operation = Operation::Decode;
        let input = self.token_content.text();
        match jsonwebtoken::decode_header(input.as_bytes()) {
            Ok(header) => {
                replace_text_in_field(
                    &mut self.header_content,
                    serde_json::to_string_pretty(&header).unwrap(),
                );
                self.algorithm = header.alg;

                match jsonwebtoken::dangerous::insecure_decode::<serde_json::Value>(
                    input.as_bytes(),
                ) {
                    Ok(token_data) => {
                        replace_text_in_field(
                            &mut self.claims_content,
                            serde_json::to_string_pretty(&token_data.claims).unwrap(),
                        );
                        let decoding_key;
                        match algorithm_type(self.algorithm) {
                            AlgorithmType::Symmetric => {
                                if self.key_encoding == 0 {
                                    decoding_key = jsonwebtoken::DecodingKey::from_secret(
                                        self.symmetric_key.as_bytes(),
                                    );
                                } else {
                                    if let Ok(key) = jsonwebtoken::DecodingKey::from_base64_secret(
                                        &self.symmetric_key.as_str(),
                                    ) {
                                        decoding_key = key;
                                    } else {
                                        self.status = "invalid-key".to_string();
                                        return;
                                    }
                                };
                            }
                            AlgorithmType::Asymmetric => {
                                let decoding_key_result = match self.algorithm {
                                    Algorithm::RS256
                                    | Algorithm::RS384
                                    | Algorithm::RS512
                                    | Algorithm::PS256
                                    | Algorithm::PS384
                                    | Algorithm::PS512 => jsonwebtoken::DecodingKey::from_rsa_pem(
                                        self.public_key_content.text().as_bytes(),
                                    ),
                                    Algorithm::ES256 | Algorithm::ES384 => {
                                        jsonwebtoken::DecodingKey::from_ec_pem(
                                            self.public_key_content.text().as_bytes(),
                                        )
                                    }
                                    Algorithm::EdDSA => jsonwebtoken::DecodingKey::from_ed_pem(
                                        self.public_key_content.text().as_bytes(),
                                    ),
                                    _ => Err(jsonwebtoken::errors::new_error(
                                        jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
                                    )),
                                };
                                if let Ok(key) = decoding_key_result {
                                    decoding_key = key;
                                } else {
                                    self.status = "invalid-key".to_string();
                                    return;
                                }
                            }
                        };

                        let mut validation = jsonwebtoken::Validation::new(self.algorithm);
                        validation.validate_aud = false;
                        validation.validate_exp = false;
                        validation.set_required_spec_claims::<&str>(&[]);
                        match jsonwebtoken::decode::<serde_json::Value>(
                            input.as_bytes(),
                            &decoding_key,
                            &validation,
                        ) {
                            Ok(_) => self.status = "ok".to_string(),
                            Err(_) => self.status = "invalid-signature".to_string(),
                        };
                    }
                    Err(_) => {
                        self.status = "invalid-token".to_string();
                    }
                };
            }
            Err(_) => {
                self.status = "invalid-token".to_string();
            }
        };
    }

    fn encode_token(&mut self) {
        self.last_operation = Operation::Encode;
        let header_input = self.header_content.text();
        match serde_json::from_str::<serde_json::Value>(header_input.as_str()) {
            Ok(header_value) => {
                if header_value.is_object() {
                    let header_object = header_value.as_object().unwrap();
                    if header_object.contains_key("alg") {
                        match serde_json::from_value::<jsonwebtoken::Header>(header_value) {
                            Ok(header) => {
                                self.algorithm = header.alg;
                                let claims_input = self.claims_content.text();
                                match serde_json::from_str::<serde_json::Value>(
                                    claims_input.as_str(),
                                ) {
                                    Ok(claims_value) => {
                                        if claims_value.is_object() {
                                            let encoding_key;
                                            match algorithm_type(self.algorithm) {
                                                AlgorithmType::Symmetric => {
                                                    if self.key_encoding == 0 {
                                                        encoding_key =
                                                            jsonwebtoken::EncodingKey::from_secret(
                                                                self.symmetric_key.as_bytes(),
                                                            );
                                                    } else {
                                                        if let Ok(key) =
                                                    jsonwebtoken::EncodingKey::from_base64_secret(
                                                        &self.symmetric_key.as_str(),
                                                    )
                                                {
                                                    encoding_key = key;
                                                } else {
                                                    self.status = "invalid-key".to_string();
                                                    return;
                                                }
                                                    }
                                                }
                                                AlgorithmType::Asymmetric => {
                                                    let encoding_key_result = match self.algorithm {
                                                        Algorithm::RS256
                                                        | Algorithm::RS384
                                                        | Algorithm::RS512
                                                        | Algorithm::PS256
                                                        | Algorithm::PS384
                                                        | Algorithm::PS512 => jsonwebtoken::EncodingKey::from_rsa_pem(
                                                            self.private_key_content.text().as_bytes(),
                                                        ),
                                                        Algorithm::ES256 | Algorithm::ES384 => {
                                                            jsonwebtoken::EncodingKey::from_ec_pem(
                                                                self.private_key_content.text().as_bytes(),
                                                            )
                                                        }
                                                        Algorithm::EdDSA => jsonwebtoken::EncodingKey::from_ed_pem(
                                                            self.private_key_content.text().as_bytes(),
                                                        ),
                                                        _ => Err(jsonwebtoken::errors::new_error(
                                                            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
                                                        )),
                                                    };
                                                    if let Ok(key) = encoding_key_result {
                                                        encoding_key = key;
                                                    } else {
                                                        self.status = "invalid-key".to_string();
                                                        return;
                                                    }
                                                }
                                            }
                                            match jsonwebtoken::encode(
                                                &header,
                                                &claims_value,
                                                &encoding_key,
                                            ) {
                                                Ok(token) => {
                                                    replace_text_in_field(
                                                        &mut self.token_content,
                                                        token,
                                                    );
                                                    self.status = "ok".to_string();
                                                }
                                                Err(err) => {
                                                    println!("{:?}", err);
                                                    self.status = "encoding-failed".to_string();
                                                }
                                            }
                                        } else {
                                            self.status = "invalid-claims".to_string();
                                        }
                                    }
                                    Err(_) => self.status = "invalid-claims".to_string(),
                                }
                            }
                            Err(_) => {
                                self.status = "invalid-header".to_string();
                            }
                        }
                    } else {
                        self.status = "invalid-header".to_string();
                    }
                } else {
                    self.status = "invalid-header".to_string();
                }
            }
            Err(_) => self.status = "invalid-header".to_string(),
        }
    }

    fn perform_last_operation(&mut self) {
        match self.last_operation {
            Operation::Decode => self.decode_token(),
            Operation::Encode => self.encode_token(),
        };
    }
}
