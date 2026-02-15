use {
    crate::{Message, app::AppModel, fl, i18n::LANGUAGE_LOADER, utility_pages::UtilityPage},
    chrono::{DateTime, Datelike, FixedOffset, Local},
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, clipboard,
            widget::{column, row},
        },
        iced_widget,
        widget::{self, Id, TextInput, text_input},
    },
};

const UNIX_TEXT_ID: &str = "unix-text";
const ISO_8601_TEXT_ID: &str = "iso-8601-text";
const EMAIL_TEXT_ID: &str = "email-text";
const DMY_TEXT_ID: &str = "dmy-text";
const HR_TEXT_ID: &str = "hr-text";
const TIMESTAMP_TYPES: [&str; 2] = ["epoch-seconds", "epoch-milliseconds"];

/// Offset seconds east of UTC
const UTC_OFFSET_SECONDS: &[i32; 38] = &[
    -12 * 3600,
    -11 * 3600,
    -10 * 3600,
    -9 * 3600 - 30 * 60,
    -9 * 3600,
    -8 * 3600,
    -7 * 3600,
    -6 * 3600,
    -5 * 3600,
    -4 * 3600,
    -3 * 3600 - 30 * 60,
    -3 * 3600,
    -2 * 3600,
    -1 * 3600,
    0,
    1 * 3600,
    2 * 3600,
    3 * 3600,
    3 * 3600 + 30 * 60,
    4 * 3600,
    4 * 3600 + 30 * 60,
    5 * 3600,
    5 * 3600 + 30 * 60,
    5 * 3600 + 45 * 60,
    6 * 3600,
    6 * 3600 + 30 * 60,
    7 * 3600,
    8 * 3600,
    8 * 3600 + 45 * 60,
    9 * 3600,
    9 * 3600 + 30 * 60,
    10 * 3600,
    10 * 3600 + 30 * 60,
    11 * 3600,
    12 * 3600,
    12 * 3600 + 45 * 60,
    13 * 3600,
    14 * 3600,
];

#[derive(Debug, Clone)]
pub enum UnixTimeConverterMessage {
    InputTextChanged(String),
    TimestampTypeChanged(usize),
    TimezoneChanged(usize),
    SetCurrentTime,
    UnixTimestamp(),
    SelectAllTextField(Id),
    CopyText(Id),
    ReadOnlyInput(Id),
}

pub(crate) struct UnixTimeConverterPage {
    text: String,
    iso_8601_time: String,
    email_time: String,
    dmy_time: String,
    hr_time: String,
    timestamp_type: usize,
    selected_timezone: usize,
    timezones: Vec<String>,
}

impl Default for UnixTimeConverterPage {
    fn default() -> Self {
        UnixTimeConverterPage {
            text: String::default(),
            iso_8601_time: String::default(),
            email_time: String::default(),
            dmy_time: String::default(),
            hr_time: String::default(),
            timestamp_type: 0,
            selected_timezone: find_current_timezone_position(),
            timezones: UTC_OFFSET_SECONDS
                .map(|offset| format_offset(FixedOffset::east_opt(offset.clone()).unwrap()))
                .to_vec(),
        }
    }
}

fn find_current_timezone_position() -> usize {
    let current_offset = chrono::offset::Local::now().offset().local_minus_utc();
    match UTC_OFFSET_SECONDS.binary_search(&current_offset) {
        Ok(position) => position,
        Err(position) => position,
    }
}

fn clipboard_button(id: &str) -> Element<'_, Message> {
    widget::tooltip(
        widget::button::icon(widget::icon::from_name("edit-copy-symbolic")).on_press(
            Message::UnixTimeConverterMessage(UnixTimeConverterMessage::CopyText(Id::new(
                id.to_string(),
            ))),
        ),
        widget::text(fl!("copy")),
        iced_widget::tooltip::Position::Bottom,
    )
    .into()
}

pub fn format_offset(offset: FixedOffset) -> String {
    let secs = offset.local_minus_utc();

    let sign = if secs >= 0 { '+' } else { '-' };
    let secs = secs.abs();

    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;

    format!("UTC{}{:02}:{:02}", sign, hours, minutes)
}

impl UtilityPage for UnixTimeConverterPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;
        let header = widget::row::with_capacity(2)
            .push(widget::text::title2(fl!("unix-time-converter")))
            .align_y(Alignment::End)
            .spacing(space_s);

        let input_header: Element<'_, Message> = widget::text::title4(fl!("input"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let timestamp_option: Element<'_, Message> = widget::settings::item(
            fl!("unix-time-converter", "unix-timestamp"),
            widget::dropdown(
                TIMESTAMP_TYPES
                    .map(|timestamp_type| {
                        LANGUAGE_LOADER.get_attr("unix-time-converter", timestamp_type)
                    })
                    .to_vec(),
                Some(self.timestamp_type),
                |selection| {
                    Message::UnixTimeConverterMessage(
                        UnixTimeConverterMessage::TimestampTypeChanged(selection),
                    )
                },
            ),
        )
        .into();

        let timestamp_input: TextInput<'_, Message> = widget::text_input("", &self.text)
            .id(Id::new(UNIX_TEXT_ID))
            .trailing_icon(
                row![
                    widget::button::text(fl!("unix-time-converter", "now")).on_press(
                        Message::UnixTimeConverterMessage(UnixTimeConverterMessage::SetCurrentTime)
                    ),
                    clipboard_button(UNIX_TEXT_ID),
                ]
                .into(),
            )
            .on_input(|text| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::InputTextChanged(text))
            })
            .on_submit(|_| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::UnixTimestamp())
            });

        let output_header: Element<'_, Message> = widget::text::title4(fl!("output"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let timezone_option: Element<'_, Message> = widget::settings::item(
            fl!("unix-time-converter", "timezone"),
            widget::dropdown(&self.timezones, Some(self.selected_timezone), |selection| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::TimezoneChanged(
                    selection,
                ))
            }),
        )
        .into();

        let iso_8601_text = widget::text_input::text_input("", &self.iso_8601_time)
            .id(Id::new(ISO_8601_TEXT_ID))
            .helper_text(fl!("unix-time-converter", "iso-8601-date"))
            .editing(false)
            .trailing_icon(clipboard_button(ISO_8601_TEXT_ID))
            .on_input(|_| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::ReadOnlyInput(Id::new(
                    ISO_8601_TEXT_ID,
                )))
            })
            .select_on_focus(true)
            .on_focus(Message::UnixTimeConverterMessage(
                UnixTimeConverterMessage::SelectAllTextField(Id::new(ISO_8601_TEXT_ID)),
            ));

        let email_text = widget::text_input::text_input("", &self.email_time)
            .id(Id::new(EMAIL_TEXT_ID))
            .helper_text(fl!("unix-time-converter", "email-date"))
            .editing(false)
            .trailing_icon(clipboard_button(EMAIL_TEXT_ID))
            .on_input(|_| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::ReadOnlyInput(Id::new(
                    EMAIL_TEXT_ID,
                )))
            })
            .select_on_focus(true)
            .on_focus(Message::UnixTimeConverterMessage(
                UnixTimeConverterMessage::SelectAllTextField(Id::new(EMAIL_TEXT_ID)),
            ));

        let dmy_text = widget::text_input::text_input("", &self.dmy_time)
            .id(Id::new(DMY_TEXT_ID))
            .helper_text("DD/MM/YYYY")
            .editing(false)
            .trailing_icon(clipboard_button(DMY_TEXT_ID))
            .on_input(|_| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::ReadOnlyInput(Id::new(
                    DMY_TEXT_ID,
                )))
            })
            .select_on_focus(true)
            .on_focus(Message::UnixTimeConverterMessage(
                UnixTimeConverterMessage::SelectAllTextField(Id::new(DMY_TEXT_ID)),
            ));

        let hr_text = widget::text_input::text_input("", &self.hr_time)
            .id(Id::new(HR_TEXT_ID))
            .helper_text(fl!("unix-time-converter", "human-readable-time"))
            .editing(false)
            .trailing_icon(clipboard_button(HR_TEXT_ID))
            .on_input(|_| {
                Message::UnixTimeConverterMessage(UnixTimeConverterMessage::ReadOnlyInput(Id::new(
                    HR_TEXT_ID,
                )))
            })
            .select_on_focus(true)
            .on_focus(Message::UnixTimeConverterMessage(
                UnixTimeConverterMessage::SelectAllTextField(Id::new(HR_TEXT_ID)),
            ));

        column![
            header,
            input_header,
            timestamp_option,
            timestamp_input,
            output_header,
            timezone_option,
            iso_8601_text,
            email_text,
            dmy_text,
            hr_text
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
            Message::UnixTimeConverterMessage(unix_time_converter_message) => {
                match unix_time_converter_message {
                    UnixTimeConverterMessage::InputTextChanged(text) => {
                        self.text = text;
                    }
                    UnixTimeConverterMessage::TimestampTypeChanged(selection) => {
                        if let Ok(time) = self.text.parse::<i64>() {
                            let new_time = match self.timestamp_type {
                                0 => {
                                    if selection == 0 {
                                        time
                                    } else {
                                        time * 1000
                                    }
                                }
                                1.. => {
                                    if selection == 0 {
                                        time / 1000
                                    } else {
                                        time
                                    }
                                }
                            };
                            self.text = new_time.to_string();
                        }
                        self.timestamp_type = selection;
                        self.convert_unix_timestamp();
                    }
                    UnixTimeConverterMessage::TimezoneChanged(selection) => {
                        self.selected_timezone = selection;
                        self.convert_unix_timestamp();
                    }
                    UnixTimeConverterMessage::SetCurrentTime => {
                        self.text = match self.timestamp_type {
                            0 => Local::now().timestamp(),
                            1.. => Local::now().timestamp_millis(),
                        }
                        .to_string();
                        self.convert_unix_timestamp();
                    }
                    UnixTimeConverterMessage::UnixTimestamp() => {
                        self.convert_unix_timestamp();
                    }
                    UnixTimeConverterMessage::SelectAllTextField(id) => {
                        return text_input::select_all(id);
                    }
                    UnixTimeConverterMessage::CopyText(id) => {
                        if id == Id::new(UNIX_TEXT_ID) {
                            return clipboard::write(self.text.clone());
                        } else if id == Id::new(EMAIL_TEXT_ID) {
                            return clipboard::write(self.email_time.clone());
                        } else if id == Id::new(ISO_8601_TEXT_ID) {
                            return clipboard::write(self.iso_8601_time.clone());
                        }
                    }
                    UnixTimeConverterMessage::ReadOnlyInput(id) => {
                        return text_input::move_cursor_to_end(id);
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

impl UnixTimeConverterPage {
    fn convert_unix_timestamp(&mut self) {
        let _ = self.text.parse::<i64>().inspect(|time| {
            match self.timestamp_type {
                0 => DateTime::from_timestamp_secs(*time),
                1.. => DateTime::from_timestamp_millis(*time),
            }
            .inspect(|date_time| {
                let date_time_with_offset = date_time.with_timezone(
                    &FixedOffset::east_opt(UTC_OFFSET_SECONDS[self.selected_timezone]).unwrap(),
                );

                self.iso_8601_time = date_time_with_offset.to_rfc3339();

                if date_time_with_offset.year() >= 0 && date_time_with_offset.year() < 10000 {
                    self.email_time = date_time_with_offset.to_rfc2822();
                } else {
                    self.email_time = String::new();
                }

                self.dmy_time.clear();
                let _ = date_time_with_offset
                    .format("%d/%m/%Y")
                    .write_to(&mut self.dmy_time);

                self.hr_time.clear();
                let _ = date_time_with_offset
                    .format("%b %e, %l:%M %p")
                    .write_to(&mut self.hr_time);
            });
        });
    }
}
