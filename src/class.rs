use cosmic::{iced_core::Color, prelude::ColorExt, widget::text_input::Appearance};

pub(crate) fn text_editor_class(
    theme: &cosmic::Theme,
    status: cosmic::widget::text_editor::Status,
) -> cosmic::iced_widget::text_editor::Style {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: cosmic::iced::Color = container.component.base.into();
    background.a = 0.25;
    let selection = cosmic.accent.base.into();
    let value = cosmic.palette.neutral_9.into();
    let mut placeholder = cosmic.palette.neutral_9;
    placeholder.alpha = 0.7;
    let placeholder = placeholder.into();

    match status {
        cosmic::iced_widget::text_editor::Status::Active
        | cosmic::iced_widget::text_editor::Status::Disabled => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: container.component.divider.into(),
                },
                placeholder,
                value,
                selection,
            }
        }
        cosmic::iced_widget::text_editor::Status::Hovered
        | cosmic::iced_widget::text_editor::Status::Focused { .. } => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: cosmic::iced::Color::from(cosmic.accent.base),
                },
                placeholder,
                value,
                selection,
            }
        }
    }
}

fn text_input_default_appearance(theme: &cosmic::Theme) -> cosmic::widget::text_input::Appearance {
    let palette = theme.cosmic();
    let container = theme.current_container();

    let mut background: Color = container.component.base.into();
    background.a = 0.25;

    let corner = palette.corner_radii;
    let label_color = palette.palette.neutral_9;

    Appearance {
        background: background.into(),
        border_radius: corner.radius_s.into(),
        border_width: 1.0,
        border_offset: None,
        border_color: container.component.divider.into(),
        icon_color: None,
        text_color: None,
        placeholder_color: {
            let color: Color = container.on.into();
            color.blend_alpha(background, 0.7)
        },
        selected_text_color: palette.on_accent_color().into(),
        selected_fill: palette.accent_color().into(),
        label_color: label_color.into(),
    }
}

fn text_input_focused_style(theme: &cosmic::Theme) -> cosmic::widget::text_input::Appearance {
    let mut appearance = text_input_default_appearance(theme);
    appearance.border_color = cosmic::iced::Color::from(theme.cosmic().accent.base);
    appearance
}

pub(crate) fn text_input_style() -> cosmic::theme::TextInput {
    cosmic::theme::TextInput::Custom {
        active: Box::new(text_input_default_appearance),
        error: Box::new(text_input_default_appearance),
        hovered: Box::new(text_input_focused_style),
        focused: Box::new(text_input_focused_style),
        disabled: Box::new(text_input_default_appearance),
    }
}
