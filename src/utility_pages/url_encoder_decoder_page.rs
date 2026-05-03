use {
    crate::{
        Message,
        app::AppModel,
        components::{
            copy_button, input_text_editor, output_text_editor, page_header, paste_button,
            status_bar,
        },
        fl,
        i18n::LANGUAGE_LOADER,
        utility_pages::UtilityPage,
    },
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, clipboard,
            widget::{column, row},
        },
        widget::{self, Id, text_editor},
    },
    std::sync::Arc,
    urlencoding,
};

const INPUT_EDITOR_ID: &str = "input-editor";
const OUTPUT_EDITOR_ID: &str = "output-editor";
const OPERATIONS: [&str; 2] = ["encode", "decode"];

#[derive(Debug, Clone)]
pub enum UrlEncoderDecoderMessage {
    InputEditorAction(text_editor::Action),
    OutputEditorAction(text_editor::Action),
    OperationChanged(usize),
    ConvertInput,
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    NoOp,
}

pub(crate) struct UrlEncoderDecoderPage {
    input_content: text_editor::Content,
    output_content: text_editor::Content,
    selected_operation: usize,
    status: String,
}

impl Default for UrlEncoderDecoderPage {
    fn default() -> Self {
        UrlEncoderDecoderPage {
            input_content: text_editor::Content::default(),
            output_content: text_editor::Content::default(),
            selected_operation: 1,
            status: "ok".to_string(),
        }
    }
}

impl UtilityPage for UrlEncoderDecoderPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = page_header(UrlEncoderDecoderPage::PAGE_NAME);

        let input_header: Element<'_, Message> = row![
            widget::text::title4(fl!("input"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::button::text(fl!("convert")).on_press(Message::UrlEncoderDecoderMessage(
                UrlEncoderDecoderMessage::ConvertInput
            )),
            widget::dropdown(
                OPERATIONS
                    .map(|operation| LANGUAGE_LOADER.get(operation))
                    .to_vec(),
                Some(self.selected_operation),
                |selection| {
                    Message::UrlEncoderDecoderMessage(UrlEncoderDecoderMessage::OperationChanged(
                        selection,
                    ))
                },
            ),
            paste_button(Message::UrlEncoderDecoderMessage(
                UrlEncoderDecoderMessage::PasteText(Id::new(INPUT_EDITOR_ID))
            )),
        ]
        .into();

        let input_editor: Element<'_, Message> = input_text_editor(
            &self.input_content,
            |action| {
                Message::UrlEncoderDecoderMessage(UrlEncoderDecoderMessage::InputEditorAction(
                    action,
                ))
            },
            None,
            None::<u32>,
        );

        let output_header: Element<'_, Message> = row![
            widget::text::title4(fl!("output"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            copy_button(Message::UrlEncoderDecoderMessage(
                UrlEncoderDecoderMessage::CopyText(Id::new(OUTPUT_EDITOR_ID))
            ))
        ]
        .align_y(Alignment::Center)
        .into();

        let output_editor: Element<'_, Message> = output_text_editor(
            &self.output_content,
            |action| {
                Message::UrlEncoderDecoderMessage(UrlEncoderDecoderMessage::OutputEditorAction(
                    action,
                ))
            },
            None,
            None::<u32>,
        );

        let status_bar = status_bar(self.status.as_str(), UrlEncoderDecoderPage::PAGE_NAME);

        column![
            header,
            input_header,
            input_editor,
            output_header,
            output_editor,
            status_bar
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
            Message::UrlEncoderDecoderMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    UrlEncoderDecoderMessage::InputEditorAction(action) => {
                        self.input_content.perform(action);
                    }
                    UrlEncoderDecoderMessage::OutputEditorAction(action) => {
                        if !action.is_edit() {
                            self.output_content.perform(action);
                        }
                    }
                    UrlEncoderDecoderMessage::OperationChanged(selection) => {
                        if selection != self.selected_operation {
                            self.selected_operation = selection;
                            self.convert_input();
                        }
                    }
                    UrlEncoderDecoderMessage::ConvertInput => {
                        self.convert_input();
                    }
                    UrlEncoderDecoderMessage::CopyText(id) => {
                        let mut to_copy: String = String::new();
                        if id == Id::new(INPUT_EDITOR_ID) {
                            to_copy = self.input_content.text();
                        } else if id == Id::new(OUTPUT_EDITOR_ID) {
                            to_copy = self.output_content.text();
                        }
                        return clipboard::write(to_copy);
                    }
                    UrlEncoderDecoderMessage::PasteText(id) => {
                        return clipboard::read().map(move |optional_data| match optional_data {
                            Some(data) => cosmic::Action::App(Message::UrlEncoderDecoderMessage(
                                UrlEncoderDecoderMessage::ReplaceText(id.clone(), data),
                            )),
                            None => cosmic::Action::App(Message::UrlEncoderDecoderMessage(
                                UrlEncoderDecoderMessage::NoOp,
                            )),
                        });
                    }
                    UrlEncoderDecoderMessage::ReplaceText(id, text) => {
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
                    UrlEncoderDecoderMessage::NoOp => {}
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl UrlEncoderDecoderPage {
    const PAGE_NAME: &str = "url-encoder-decoder";

    fn convert_input(&mut self) {
        let input = self.input_content.text();
        match self.selected_operation {
            0 => {
                self.output_content.perform(text_editor::Action::SelectAll);
                self.output_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.output_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(urlencoding::encode(input.as_str()).into_owned()),
                    )));
                self.status = "ok".to_string();
            }
            _ => {
                let decoded_url = urlencoding::decode(input.as_str());
                match decoded_url {
                    Ok(decoded_url) => {
                        self.output_content.perform(text_editor::Action::SelectAll);
                        self.output_content
                            .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                        self.output_content.perform(text_editor::Action::Edit(
                            text_editor::Edit::Paste(Arc::new(decoded_url.into_owned())),
                        ));
                        self.status = "ok".to_string();
                    }
                    Err(_) => {
                        self.status = "url-decoding-error".to_string();
                    }
                }
            }
        }
    }
}
