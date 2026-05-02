use {
    crate::{Message, fl, i18n::LANGUAGE_LOADER},
    cosmic::{
        self, Element,
        iced::{
            Alignment, Border, Length,
            widget::{container, row},
        },
        widget::{self, Tooltip},
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
            .background(theme.cosmic().bg_component_color())
            .border(Border::default().rounded(theme.cosmic().radius_s()))
    })
    .into()
}
