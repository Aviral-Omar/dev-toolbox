use {
    crate::{
        Message, app::AppModel, class::text_editor_class, fl, i18n::LANGUAGE_LOADER,
        utility_pages::UtilityPage,
    },
    base64::Engine,
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, Padding, clipboard,
            keyboard::{Key, key},
            widget::{column, row, text_editor},
        },
        iced_widget,
        widget::{self, Id, text_editor::Binding},
    },
    std::sync::Arc,
};

const INPUT_EDITOR_ID: &str = "input-editor";
const OUTPUT_EDITOR_ID: &str = "output-editor";
const OPERATIONS: [&str; 2] = ["encode", "decode"];

#[derive(Debug, Clone)]
pub enum Base64StringEncoderDecoderMessage {
    InputEditorAction(text_editor::Action),
    OutputEditorAction(text_editor::Action),
    OperationChanged(usize),
    UrlSafeToggled(bool),
    ConvertInput,
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    NoOp,
}

#[derive(Default)]
pub(crate) struct Base64StringEncoderDecoderPage {
    input_content: text_editor::Content,
    output_content: text_editor::Content,
    selected_operation: usize,
    url_safe: bool,
}

impl UtilityPage for Base64StringEncoderDecoderPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("base64-string-encoder-decoder")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let input_header: Element<'_, Message> = row![
            widget::text::title4(fl!("input"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::button::text(fl!("convert")).on_press(
                Message::Base64StringEncoderDecoderMessage(
                    Base64StringEncoderDecoderMessage::ConvertInput
                )
            ),
            widget::dropdown(
                OPERATIONS
                    .map(|operation| LANGUAGE_LOADER.get(operation))
                    .to_vec(),
                Some(self.selected_operation),
                |selection| {
                    Message::Base64StringEncoderDecoderMessage(
                        Base64StringEncoderDecoderMessage::OperationChanged(selection),
                    )
                },
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::Base64StringEncoderDecoderMessage(
                        Base64StringEncoderDecoderMessage::PasteText(Id::new(INPUT_EDITOR_ID)),
                    )
                ),
                widget::text(fl!("paste")),
                iced_widget::tooltip::Position::Bottom,
            ),
        ]
        .into();

        let input_editor: Element<'_, Message> = text_editor(&self.input_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .on_action(|action| {
                Message::Base64StringEncoderDecoderMessage(
                    Base64StringEncoderDecoderMessage::InputEditorAction(action),
                )
            })
            .key_binding(|key_press| {
                if key_press.key == Key::Named(key::Named::Tab)
                    && key_press.status == text_editor::Status::Focused
                {
                    return Some(Binding::Insert('\t'));
                }
                return Binding::from_key_press(key_press);
            })
            .into();

        let output_header: Element<'_, Message> = row![
            widget::text::title4(fl!("output"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::container(
                widget::checkbox(
                    fl!("base64-string-encoder-decoder", "url-safe"),
                    self.url_safe
                )
                .on_toggle(|url_safe| {
                    Message::Base64StringEncoderDecoderMessage(
                        Base64StringEncoderDecoderMessage::UrlSafeToggled(url_safe),
                    )
                }),
            )
            .padding(Padding::new(0.0).right(8.0)),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::Base64StringEncoderDecoderMessage(
                        Base64StringEncoderDecoderMessage::CopyText(Id::new(OUTPUT_EDITOR_ID)),
                    ),
                ),
                widget::text(fl!("copy")),
                iced_widget::tooltip::Position::Bottom,
            )
        ]
        .align_y(Alignment::Center)
        .into();

        let output_editor: Element<'_, Message> = text_editor(&self.output_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .on_action(|action| {
                Message::Base64StringEncoderDecoderMessage(
                    Base64StringEncoderDecoderMessage::OutputEditorAction(action),
                )
            })
            .into();

        column![
            header,
            input_header,
            input_editor,
            output_header,
            output_editor
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
            Message::Base64StringEncoderDecoderMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    Base64StringEncoderDecoderMessage::InputEditorAction(action) => {
                        self.input_content.perform(action);
                    }
                    Base64StringEncoderDecoderMessage::OutputEditorAction(action) => {
                        if !action.is_edit() {
                            self.output_content.perform(action);
                        }
                    }
                    Base64StringEncoderDecoderMessage::OperationChanged(selection) => {
                        if selection != self.selected_operation {
                            self.selected_operation = selection;
                            self.convert_input();
                        }
                    }
                    Base64StringEncoderDecoderMessage::UrlSafeToggled(url_safe) => {
                        self.url_safe = url_safe;
                        self.convert_input();
                    }
                    Base64StringEncoderDecoderMessage::ConvertInput => {
                        self.convert_input();
                    }
                    Base64StringEncoderDecoderMessage::CopyText(id) => {
                        // Content::text adds \n if text doesn't end with it, hence removing it
                        let mut to_copy: String = String::new();
                        if id == Id::new(INPUT_EDITOR_ID) {
                            to_copy = self.input_content.text();
                            if !self.input_content.lines().last().unwrap().is_empty() {
                                to_copy.pop();
                            }
                        } else if id == Id::new(OUTPUT_EDITOR_ID) {
                            to_copy = self.output_content.text();
                            if !self.output_content.lines().last().unwrap().is_empty() {
                                to_copy.pop();
                            }
                        }
                        return clipboard::write(to_copy);
                    }
                    Base64StringEncoderDecoderMessage::PasteText(id) => {
                        return clipboard::read().map(move |optional_data| match optional_data {
                            Some(data) => {
                                cosmic::Action::App(Message::Base64StringEncoderDecoderMessage(
                                    Base64StringEncoderDecoderMessage::ReplaceText(
                                        id.clone(),
                                        data,
                                    ),
                                ))
                            }
                            None => {
                                cosmic::Action::App(Message::Base64StringEncoderDecoderMessage(
                                    Base64StringEncoderDecoderMessage::NoOp,
                                ))
                            }
                        });
                    }
                    Base64StringEncoderDecoderMessage::ReplaceText(id, text) => {
                        if id == Id::new(INPUT_EDITOR_ID) {
                            self.input_content.perform(text_editor::Action::SelectAll);
                            self.input_content.perform(text_editor::Action::Edit(
                                text_editor::Edit::Paste(Arc::new(text)),
                            ));
                        } else if id == Id::new(OUTPUT_EDITOR_ID) {
                            self.output_content.perform(text_editor::Action::SelectAll);
                            self.output_content.perform(text_editor::Action::Edit(
                                text_editor::Edit::Paste(Arc::new(text)),
                            ));
                        }
                    }
                    Base64StringEncoderDecoderMessage::NoOp => {}
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl Base64StringEncoderDecoderPage {
    fn convert_input(&mut self) {
        let engine = if self.url_safe {
            base64::engine::general_purpose::URL_SAFE
        } else {
            base64::engine::general_purpose::STANDARD
        };
        let mut input = self.input_content.text();
        if !self.input_content.lines().last().unwrap().is_empty() {
            input.pop();
        }
        match self.selected_operation {
            0 => {
                self.output_content.perform(text_editor::Action::SelectAll);
                self.output_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.output_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(engine.encode(input.trim().as_bytes())),
                    )));
            }
            _ => {
                let decode_result = engine.decode(input.trim());
                if let Ok(bytes) = decode_result {
                    self.output_content.perform(text_editor::Action::SelectAll);
                    self.output_content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    self.output_content.perform(text_editor::Action::Edit(
                        text_editor::Edit::Paste(Arc::new(
                            String::from_utf8_lossy(bytes.as_slice()).into_owned(),
                        )),
                    ));
                } else if let Err(err) = decode_result {
                    println!("Base64 Decoding Error: {}", err);
                }
            }
        }
    }
}
