use {
    crate::{Message, app::AppModel, class::text_editor_class, fl, utility_pages::UtilityPage},
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
    quick_xml,
    serde::Serialize,
    serde_json, serde_saphyr,
    std::sync::Arc,
    toml,
};

const INPUT_EDITOR_ID: &str = "input-editor";
const OUTPUT_EDITOR_ID: &str = "output-editor";
const DATA_FORMATS: [&str; 4] = ["JSON", "YAML", "XML", "TOML"];
const INDENTS: [&str; 3] = ["2", "4", "8"];

#[derive(Debug, Clone)]
pub enum DataConverterFormatterMessage {
    InputEditorAction(text_editor::Action),
    OutputEditorAction(text_editor::Action),
    InputFormatChanged(usize),
    OutputFormatChanged(usize),
    IndentChanged(usize),
    ConvertInput,
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    NoOp,
}

#[derive(Default)]
pub(crate) struct DataConverterFormatterPage {
    input_content: text_editor::Content,
    output_content: text_editor::Content,
    input_format: usize,
    output_format: usize,
    selected_indent: usize,
}

impl UtilityPage for DataConverterFormatterPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("data-converter-formatter")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let input_header: Element<'_, Message> = row![
            widget::text::title4(fl!("input"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::button::text(fl!("convert")).on_press(Message::DataConverterFormatterMessage(
                DataConverterFormatterMessage::ConvertInput
            )),
            widget::dropdown(
                DATA_FORMATS.to_vec(),
                Some(self.input_format),
                |selection| {
                    Message::DataConverterFormatterMessage(
                        DataConverterFormatterMessage::InputFormatChanged(selection),
                    )
                },
            ),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(
                    Message::DataConverterFormatterMessage(
                        DataConverterFormatterMessage::PasteText(Id::new(INPUT_EDITOR_ID)),
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
                Message::DataConverterFormatterMessage(
                    DataConverterFormatterMessage::InputEditorAction(action),
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

        let output_header: Element<'_, Message> = widget::row()
            .push(
                widget::text::title4(fl!("output"))
                    .width(Length::Fill)
                    .align_x(Alignment::Start),
            )
            .push_maybe(if self.output_format != 3 {
                Some(widget::dropdown(
                    INDENTS
                        .map(|indent| {
                            format!("{} {}", indent, fl!("data-converter-formatter", "spaces"))
                        })
                        .to_vec(),
                    Some(self.selected_indent),
                    |selection| {
                        Message::DataConverterFormatterMessage(
                            DataConverterFormatterMessage::IndentChanged(selection),
                        )
                    },
                ))
            } else {
                None
            })
            .push(widget::dropdown(
                DATA_FORMATS.to_vec(),
                Some(self.output_format),
                |selection| {
                    Message::DataConverterFormatterMessage(
                        DataConverterFormatterMessage::OutputFormatChanged(selection),
                    )
                },
            ))
            .push(widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::DataConverterFormatterMessage(
                        DataConverterFormatterMessage::CopyText(Id::new(OUTPUT_EDITOR_ID)),
                    ),
                ),
                widget::text(fl!("copy")),
                iced_widget::tooltip::Position::Bottom,
            ))
            .into();

        let output_editor: Element<'_, Message> = text_editor(&self.output_content)
            .padding(Padding::new(12.0))
            .height(Length::Fill)
            .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                text_editor_class,
            )))
            .on_action(|action| {
                Message::DataConverterFormatterMessage(
                    DataConverterFormatterMessage::OutputEditorAction(action),
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
            Message::DataConverterFormatterMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    DataConverterFormatterMessage::InputEditorAction(action) => {
                        self.input_content.perform(action);
                    }
                    DataConverterFormatterMessage::OutputEditorAction(action) => {
                        if !action.is_edit() {
                            self.output_content.perform(action);
                        }
                    }
                    DataConverterFormatterMessage::InputFormatChanged(selection) => {
                        self.input_format = selection
                    }
                    DataConverterFormatterMessage::OutputFormatChanged(selection) => {
                        self.output_format = selection;
                        self.convert_input();
                    }
                    DataConverterFormatterMessage::IndentChanged(selection) => {
                        self.selected_indent = selection;
                        self.convert_input();
                    }
                    DataConverterFormatterMessage::ConvertInput => {
                        self.convert_input();
                    }
                    DataConverterFormatterMessage::CopyText(id) => {
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
                    DataConverterFormatterMessage::PasteText(id) => {
                        return clipboard::read().map(move |optional_data| match optional_data {
                            Some(data) => {
                                cosmic::Action::App(Message::DataConverterFormatterMessage(
                                    DataConverterFormatterMessage::ReplaceText(id.clone(), data),
                                ))
                            }
                            None => cosmic::Action::App(Message::DataConverterFormatterMessage(
                                DataConverterFormatterMessage::NoOp,
                            )),
                        });
                    }
                    DataConverterFormatterMessage::ReplaceText(id, text) => {
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
                    DataConverterFormatterMessage::NoOp => {}
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl DataConverterFormatterPage {
    fn convert_input(&mut self) {
        let mut input = self.input_content.text();
        if !self.input_content.lines().last().unwrap().is_empty() {
            input.pop();
        }
        let input_value: Option<serde_json::Value> = match self.input_format {
            0 => serde_json::from_str(input.as_str()).ok(),
            1 => serde_saphyr::from_str(input.as_str()).ok(),
            2 => quick_xml::de::from_str(input.as_str()).ok(),
            _ => toml::from_str(input.as_str()).ok(),
        };
        if let Some(value) = input_value {
            self.output_content.perform(text_editor::Action::SelectAll);
            let mut output_string: String;
            let indent_count = INDENTS[self.selected_indent].parse::<usize>().unwrap();
            match self.output_format {
                0 => {
                    let mut buf = vec![];
                    let indent = " ".repeat(indent_count);
                    let formatter =
                        serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                    let mut json_serializer =
                        serde_json::Serializer::with_formatter(&mut buf, formatter);
                    value.serialize(&mut json_serializer).unwrap();
                    output_string = String::from_utf8(buf).unwrap();
                }
                1 => {
                    output_string = String::new();
                    let mut yaml_serializer = serde_saphyr::ser::YamlSerializer::with_indent(
                        &mut output_string,
                        indent_count,
                    );
                    value.serialize(&mut yaml_serializer).unwrap();
                }
                2 => {
                    output_string = String::new();
                    let mut xml_serializer =
                        quick_xml::se::Serializer::with_root(&mut output_string, Some("root"))
                            .unwrap();
                    xml_serializer.indent(' ', indent_count);
                    xml_serializer.empty_element_handling(
                        quick_xml::se::EmptyElementHandling::SelfClosedWithSpace,
                    );
                    if let Some(err) = value.serialize(xml_serializer).err() {
                        println!("Error while converting XML: {}", err);
                    }
                }
                _ => output_string = toml::to_string_pretty(&value).unwrap(),
            }
            self.output_content
                .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                    Arc::new(output_string),
                )));
        }
    }
}
