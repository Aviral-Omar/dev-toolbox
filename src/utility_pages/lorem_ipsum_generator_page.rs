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
        self, Application, Element, Task, iced,
        iced::{
            Alignment, Length, Padding, clipboard,
            widget::{column, row},
        },
        widget::{
            self, Id,
            text_editor::{self, TextEditor},
        },
    },
    lipsum::{lipsum_with_rng, lipsum_words_with_rng},
    rand::{self, Rng},
    std::sync::Arc,
};

const OUTPUT_EDITOR_ID: &str = "output-editor";
const UNITS: [&str; 3] = ["words", "sentences", "paragraphs"];

#[derive(Debug, Clone)]
pub enum LoremIpsumGeneratorMessage {
    OutputEditorAction(text_editor::Action),
    UnitChanged(usize),
    AmountChanged(String),
    CopyText(Id),
}

pub(crate) struct LoremIpsumGeneratorPage {
    output_content: text_editor::Content,
    selected_unit: usize,
    selected_amount: String,
}

impl Default for LoremIpsumGeneratorPage {
    fn default() -> Self {
        LoremIpsumGeneratorPage {
            output_content: text_editor::Content::default(),
            selected_unit: 0,
            selected_amount: "20".to_string(),
        }
    }
}

impl UtilityPage for LoremIpsumGeneratorPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("lorem-ipsum-generator")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let options_header: Element<'_, Message> = widget::text::title4(fl!("options"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let amount_option: Element<'_, Message> = widget::settings::item(
            fl!("lorem-ipsum-generator", "amount"),
            column![row![
                widget::text_input::text_input("", &self.selected_amount)
                    .style(text_input_style())
                    .width(80)
                    .on_input(|input| {
                        Message::LoremIpsumGeneratorMessage(
                            LoremIpsumGeneratorMessage::AmountChanged(input),
                        )
                    }),
                widget::dropdown(
                    UNITS
                        .map(|operation| LANGUAGE_LOADER
                            .get_attr("lorem-ipsum-generator", operation))
                        .to_vec(),
                    Some(self.selected_unit),
                    |selection| {
                        Message::LoremIpsumGeneratorMessage(
                            LoremIpsumGeneratorMessage::UnitChanged(selection),
                        )
                    },
                ),
            ]]
            .align_x(Alignment::End),
        )
        .into();

        let output_header: Element<'_, Message> = row![
            widget::text::title4(fl!("output"))
                .width(Length::Fill)
                .align_x(Alignment::Start),
            widget::tooltip(
                widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
                    Message::LoremIpsumGeneratorMessage(LoremIpsumGeneratorMessage::CopyText(
                        Id::new(OUTPUT_EDITOR_ID)
                    ),),
                ),
                widget::text(fl!("copy")),
                widget::tooltip::Position::Bottom,
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
            .wrapping(iced::core::text::Wrapping::WordOrGlyph)
            .on_action(|action| {
                Message::LoremIpsumGeneratorMessage(LoremIpsumGeneratorMessage::OutputEditorAction(
                    action,
                ))
            })
            .into();

        column![
            header,
            options_header,
            amount_option,
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
            Message::LoremIpsumGeneratorMessage(data_converter_formatter_message) => {
                match data_converter_formatter_message {
                    LoremIpsumGeneratorMessage::OutputEditorAction(action) => {
                        if !action.is_edit() {
                            self.output_content.perform(action);
                        }
                    }
                    LoremIpsumGeneratorMessage::UnitChanged(selection) => {
                        if selection != self.selected_unit {
                            self.selected_unit = selection;
                            self.generate_lorem();
                        }
                    }
                    LoremIpsumGeneratorMessage::AmountChanged(amount) => {
                        self.selected_amount = amount;
                        self.generate_lorem();
                    }
                    LoremIpsumGeneratorMessage::CopyText(_) => {
                        let to_copy = self.output_content.text();
                        return clipboard::write(to_copy);
                    }
                }
            }
            _ => {
                println!("Wrong message type");
            }
        }
        Task::none()
    }
}

impl LoremIpsumGeneratorPage {
    fn generate_lorem(&mut self) {
        let amount: usize = match self.selected_amount.parse::<usize>() {
            Ok(amt) => amt,
            Err(_) => {
                return;
            }
        };
        self.output_content.perform(text_editor::Action::SelectAll);
        let lorem: String = match self.selected_unit {
            0 => generate_words(amount),
            1 => generate_sentences(amount),
            _ => generate_paragraphs(amount),
        };
        self.output_content
            .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                Arc::new(lorem),
            )));
    }
}

pub fn generate_words(n: usize) -> String {
    lipsum_with_rng(rand::thread_rng(), n)
}

pub fn generate_sentences(n: usize) -> String {
    let mut sentences = Vec::with_capacity(n);

    // An average sentence is ~15-20 words so this should make 2 sentences.
    while sentences.len() < n {
        let chunk = if sentences.is_empty() {
            lipsum_with_rng(rand::thread_rng(), 25)
        } else {
            lipsum_words_with_rng(rand::thread_rng(), 25)
        };

        let parts: Vec<&str> = chunk
            .split_inclusive(|c| c == '.' || c == '!' || c == '?')
            .collect();

        for part in parts {
            if sentences.len() < n {
                sentences.push(part.trim().to_string());
            }
        }
    }

    sentences.join(" ")
}

pub fn generate_paragraphs(n: usize) -> String {
    (0..n)
        .map(|i| {
            let s_count = 3 + (rand::thread_rng().r#gen::<usize>() / (usize::max_value() / 3));
            generate_sentences_custom(s_count, i == 0)
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn generate_sentences_custom(n: usize, start_with_lorem: bool) -> String {
    let mut sentences = Vec::with_capacity(n);
    while sentences.len() < n {
        let is_first = start_with_lorem && sentences.is_empty();
        let chunk = if is_first {
            lipsum_with_rng(rand::thread_rng(), 25)
        } else {
            lipsum_words_with_rng(rand::thread_rng(), 25)
        };
        let parts: Vec<&str> = chunk
            .split_inclusive(|c| c == '.' || c == '!' || c == '?')
            .collect();
        for part in parts {
            if sentences.len() < n {
                sentences.push(part.trim().to_string());
            }
        }
    }
    sentences.join(" ")
}
