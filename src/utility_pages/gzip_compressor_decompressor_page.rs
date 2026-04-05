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
            widget::{column, row},
        },
        iced_core, iced_widget,
        widget::{
            self, Id,
            text_editor::{self, Binding, TextEditor},
        },
    },
    flate2::{Compression, read::GzDecoder, write::GzEncoder},
    std::{
        io::{Read, Write},
        sync::Arc,
    },
};

const INPUT_EDITOR_ID: &str = "input-editor";
const OUTPUT_EDITOR_ID: &str = "output-editor";
const OPERATIONS: [&str; 2] = ["compress", "decompress"];

#[derive(Debug, Clone)]
pub enum GZipCompressorDecompressorMessage {
    InputEditorAction(text_editor::Action),
    OutputEditorAction(text_editor::Action),
    OperationChanged(usize),
    ConvertInput,
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    NoOp,
}

#[derive(Default)]
pub(crate) struct GZipCompressorDecompressorPage {
    input_content: text_editor::Content,
    output_content: text_editor::Content,
    selected_operation: usize,
}

impl UtilityPage for GZipCompressorDecompressorPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("gzip-compressor-decompressor")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let input_header: Element<'_, Message> = row![
            widget::text::title4(fl!("input"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::button::text(fl!("convert")).on_press(
                Message::GZipCompressorDecompressorMessage(
                    GZipCompressorDecompressorMessage::ConvertInput
                )
            ),
            widget::dropdown(
                OPERATIONS
                    .map(|operation| LANGUAGE_LOADER.get(operation))
                    .to_vec(),
                Some(self.selected_operation),
                |selection| {
                    Message::GZipCompressorDecompressorMessage(
                        GZipCompressorDecompressorMessage::OperationChanged(selection),
                    )
                },
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::GZipCompressorDecompressorMessage(
                        GZipCompressorDecompressorMessage::PasteText(Id::new(INPUT_EDITOR_ID)),
                    )
                ),
                widget::text(fl!("paste")),
                iced_widget::tooltip::Position::Bottom,
            ),
        ]
        .into();

        let input_editor: Element<'_, Message> = TextEditor::new(&self.input_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced_core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::GZipCompressorDecompressorMessage(
                    GZipCompressorDecompressorMessage::InputEditorAction(action),
                )
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

        let output_header: Element<'_, Message> = row![
            widget::text::title4(fl!("output"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::GZipCompressorDecompressorMessage(
                        GZipCompressorDecompressorMessage::CopyText(Id::new(OUTPUT_EDITOR_ID)),
                    ),
                ),
                widget::text(fl!("copy")),
                iced_widget::tooltip::Position::Bottom,
            )
        ]
        .align_y(Alignment::Center)
        .into();

        let output_editor: Element<'_, Message> = TextEditor::new(&self.output_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .wrapping(iced_core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::GZipCompressorDecompressorMessage(
                    GZipCompressorDecompressorMessage::OutputEditorAction(action),
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
            Message::GZipCompressorDecompressorMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    GZipCompressorDecompressorMessage::InputEditorAction(action) => {
                        self.input_content.perform(action);
                    }
                    GZipCompressorDecompressorMessage::OutputEditorAction(action) => {
                        if !action.is_edit() {
                            self.output_content.perform(action);
                        }
                    }
                    GZipCompressorDecompressorMessage::OperationChanged(selection) => {
                        if selection != self.selected_operation {
                            self.selected_operation = selection;
                            self.convert_input();
                        }
                    }
                    GZipCompressorDecompressorMessage::ConvertInput => {
                        self.convert_input();
                    }
                    GZipCompressorDecompressorMessage::CopyText(id) => {
                        let mut to_copy: String = String::new();
                        if id == Id::new(INPUT_EDITOR_ID) {
                            to_copy = self.input_content.text();
                        } else if id == Id::new(OUTPUT_EDITOR_ID) {
                            to_copy = self.output_content.text();
                        }
                        return clipboard::write(to_copy);
                    }
                    GZipCompressorDecompressorMessage::PasteText(id) => {
                        return clipboard::read().map(move |optional_data| match optional_data {
                            Some(data) => {
                                cosmic::Action::App(Message::GZipCompressorDecompressorMessage(
                                    GZipCompressorDecompressorMessage::ReplaceText(
                                        id.clone(),
                                        data,
                                    ),
                                ))
                            }
                            None => {
                                cosmic::Action::App(Message::GZipCompressorDecompressorMessage(
                                    GZipCompressorDecompressorMessage::NoOp,
                                ))
                            }
                        });
                    }
                    GZipCompressorDecompressorMessage::ReplaceText(id, text) => {
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
                    GZipCompressorDecompressorMessage::NoOp => {}
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl GZipCompressorDecompressorPage {
    fn convert_input(&mut self) {
        let engine = base64::engine::general_purpose::STANDARD;
        let input = self.input_content.text();
        match self.selected_operation {
            0 => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(input.as_bytes()).unwrap();
                let compressed_data = encoder.finish().unwrap();
                self.output_content.perform(text_editor::Action::SelectAll);
                self.output_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.output_content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(engine.encode(compressed_data)),
                    )));
            }
            _ => {
                let compressed_data = engine.decode(input.trim());
                if let Ok(bytes) = compressed_data {
                    let mut decoder = GzDecoder::new(&bytes[..]);
                    let mut decompressed_string = String::new();
                    match decoder.read_to_string(&mut decompressed_string) {
                        Ok(_) => {
                            self.output_content.perform(text_editor::Action::SelectAll);
                            self.output_content
                                .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                            self.output_content.perform(text_editor::Action::Edit(
                                text_editor::Edit::Paste(Arc::new(decompressed_string)),
                            ));
                        }
                        Err(err) => println!("GZip Decompression Error: {}", err),
                    };
                } else if let Err(err) = compressed_data {
                    println!("Base64 Decoding Error: {}", err);
                }
            }
        }
    }
}
