use {
    crate::{Message, class::text_editor_class, fl, i18n::LANGUAGE_LOADER},
    cosmic::{
        self, Element,
        iced::{
            Alignment, Border, Length, Padding, core,
            widget::{container, row},
        },
        widget::{self, TextEditor, Tooltip, text_editor},
    },
};

pub(crate) fn page_header(page_title: &str) -> Element<'_, Message> {
    let space_s = cosmic::theme::spacing().space_s;
    row![widget::text::title2(LANGUAGE_LOADER.get(page_title))]
        .align_y(Alignment::End)
        .spacing(space_s)
        .into()
}

pub(crate) fn copy_button<'a>(message: Message) -> Tooltip<'a, Message> {
    widget::tooltip(
        widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(message),
        widget::text(fl!("copy")),
        widget::tooltip::Position::Bottom,
    )
}

pub(crate) fn paste_button<'a>(message: Message) -> Tooltip<'a, Message> {
    widget::tooltip(
        widget::button::icon(widget::icon::from_name("edit-paste-symbolic")).on_press(message),
        widget::text(fl!("paste")),
        widget::tooltip::Position::Bottom,
    )
}

pub(crate) fn status_bar<'a>(status: &'a str, page_name: &'a str) -> Element<'a, Message> {
    widget::container(row![
        widget::text::heading(fl!("status")),
        widget::space().width(8),
        widget::text::body(LANGUAGE_LOADER.get_attr(page_name, status)),
    ])
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .padding([8, 0])
    .width(Length::Fill)
    .style(|theme| {
        container::Style::default()
            .background(theme.current_container().component.base)
            .border(Border::default().rounded(theme.cosmic().radius_s()))
    })
    .into()
}

fn custom_text_editor<'a>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
    height: Option<Length>,
    min_height: Option<impl Into<core::Pixels>>,
    tab_input: bool,
) -> Element<'a, Message> {
    let mut text_editor = TextEditor::new(content)
        .padding(Padding::new(12.0))
        .height(Length::Fill)
        .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
            text_editor_class,
        )))
        .wrapping(core::text::Wrapping::WordOrGlyph)
        .on_action(on_action);
    if let Some(height) = height {
        text_editor = text_editor.height(height);
    }
    if let Some(min_height) = min_height {
        text_editor = text_editor.min_height(min_height);
    }
    if tab_input {
        text_editor = text_editor.key_binding(|key_press| {
            if key_press.key == core::keyboard::Key::Named(core::keyboard::key::Named::Tab)
                && matches!(key_press.status, text_editor::Status::Focused { .. })
            {
                return Some(text_editor::Binding::Insert('\t'));
            }
            return text_editor::Binding::from_key_press(key_press);
        });
    }
    text_editor.into()
}

pub(crate) fn input_text_editor<'a>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
    height: Option<Length>,
    min_height: Option<impl Into<core::Pixels>>,
) -> Element<'a, Message> {
    custom_text_editor(content, on_action, height, min_height, true)
}

pub(crate) fn output_text_editor<'a>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
    height: Option<Length>,
    min_height: Option<impl Into<core::Pixels>>,
) -> Element<'a, Message> {
    custom_text_editor(content, on_action, height, min_height, false)
}
