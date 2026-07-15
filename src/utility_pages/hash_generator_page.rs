use {
    crate::{
        Message,
        app::AppModel,
        class::text_input_style,
        components::{self, input_text_editor, page_header, status_bar},
        fl,
        utility_pages::UtilityPage,
    },
    argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    },
    cosmic::{
        self, Application, Element, Task,
        iced::{
            Alignment, Length, clipboard,
            widget::{column, row},
        },
        widget::{self, Id, TextInput, text_editor, text_input},
    },
    crc32fast, hex,
    md5::{Digest, Md5, digest::DynDigest},
    murmur3::{murmur3_32, murmur3_x64_128},
    rfd::{AsyncFileDialog, FileHandle},
    rustc_hash::FxHasher,
    sha1::Sha1,
    sha2::{Sha256, Sha512},
    sha3::{Sha3_256, Sha3_512},
    std::{
        fs::File,
        hash::Hasher,
        io::{BufReader, Cursor, Read},
        sync::Arc,
    },
};

const ALGORITHMS: [&str; 13] = [
    "MD5",
    "SHA1",
    "SHA256",
    "SHA512",
    "SHA3-256",
    "SHA3-512",
    "CRC32",
    "ADLER32",
    "Murmur-32",
    "Murmur-128",
    "FxHash",
    "Bcrypt",
    "Argon2id",
];
const SEED_TEXT_ID: &str = "seed-text";
const COST_TEXT_ID: &str = "cost-text";
const SALT_TEXT_ID: &str = "salt-text";
const INPUT_EDITOR_ID: &str = "input-editor";
const HASH_TEXT_ID: &str = "hash-text";

#[derive(Debug, Clone)]
pub enum HashGeneratorMessage {
    AlgorithmChanged(usize),
    SeedTextChanged(String),
    CostTextChanged(String),
    SaltTextChanged(String),
    Randomize(Id),
    InputEditorAction(text_editor::Action),
    FileSelected(Option<FileHandle>),
    ClearContent,
    GenerateHash,
    GenerateHashResult(Result<String, String>),
    OpenFile,
    SelectAllTextField(Id),
    CopyText(Id),
    PasteText(Id),
    ReplaceText(Id, String),
    ReadOnlyInput(Id),
    NoOp,
}

pub(crate) struct HashGeneratorPage {
    algorithm: usize,
    seed: String,
    cost: String,
    salt: String,
    input_content: text_editor::Content,
    file_handle: Option<FileHandle>,
    hash_text: String,
    status: String,
}

impl Default for HashGeneratorPage {
    fn default() -> Self {
        HashGeneratorPage {
            algorithm: usize::default(),
            seed: rand::random::<u32>().to_string(),
            cost: "12".to_string(),
            salt: SaltString::generate(&mut OsRng).to_string(),
            input_content: text_editor::Content::default(),
            file_handle: Option::default(),
            hash_text: String::default(),
            status: "ok".to_string(),
        }
    }
}

fn paste_button(id: &str) -> Element<'_, Message> {
    components::paste_button(Message::HashGeneratorMessage(
        HashGeneratorMessage::PasteText(Id::new(id.to_string())),
    ))
    .into()
}

fn copy_button(id: &str) -> Element<'_, Message> {
    components::copy_button(Message::HashGeneratorMessage(
        HashGeneratorMessage::CopyText(Id::new(id.to_string())),
    ))
    .into()
}

fn randomize_button(id: &str) -> Element<'_, Message> {
    widget::tooltip(
        widget::button::icon(
            widget::icon::from_svg_bytes(include_bytes!("../../resources/icons/dice-symbolic.svg"))
                .symbolic(true),
        )
        .on_press(Message::HashGeneratorMessage(
            HashGeneratorMessage::Randomize(Id::new(id.to_string())),
        )),
        widget::text(fl!("randomize")),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

fn open_file_button<'a>() -> Element<'a, Message> {
    widget::tooltip(
        widget::button::icon(widget::icon::from_name("document-open-symbolic")).on_press(
            Message::HashGeneratorMessage(HashGeneratorMessage::OpenFile),
        ),
        widget::text(fl!("open")),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

fn clear_button<'a>() -> Element<'a, Message> {
    widget::tooltip(
        widget::button::icon(widget::icon::from_name("edit-clear-symbolic")).on_press(
            Message::HashGeneratorMessage(HashGeneratorMessage::ClearContent),
        ),
        widget::text(fl!("clear")),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

fn generate_hash_from_input(
    input: String,
    algorithm: usize,
    seed: String,
    cost: String,
    salt: String,
) -> Result<String, String> {
    match algorithm {
        0 => Ok(hex::encode(Md5::digest(input))),
        1 => Ok(hex::encode(Sha1::digest(input))),
        2 => Ok(hex::encode(Sha256::digest(input))),
        3 => Ok(hex::encode(Sha512::digest(input))),
        4 => Ok(hex::encode(Sha3_256::digest(input))),
        5 => Ok(hex::encode(Sha3_512::digest(input))),
        6 => Ok(crc32fast::hash(input.as_bytes()).to_string()),
        7 => adler2::adler32(input.as_bytes())
            .map_err(|_| "hashing-error".to_string())
            .and_then(|bytes| Ok(bytes.to_string())),
        8 => seed
            .parse::<u32>()
            .map_err(|_| "invalid-seed".to_string())
            .and_then(|seed| {
                murmur3_32(&mut Cursor::new(input), seed)
                    .map(|val| val.to_string())
                    .map_err(|_| "hashing-error".to_string())
            }),
        9 => seed
            .parse::<u32>()
            .map_err(|_| "invalid-seed".to_string())
            .and_then(|seed| {
                murmur3_x64_128(&mut Cursor::new(input), seed)
                    .map(|val| val.to_string())
                    .map_err(|_| "hashing-error".to_string())
            }),
        10 => seed
            .parse::<usize>()
            .map_err(|_| "invalid-seed".to_string())
            .map(|seed| {
                let mut hasher = FxHasher::with_seed(seed);
                hasher.write(input.as_bytes());
                hasher.finish().to_string()
            }),
        11 => cost
            .parse::<u32>()
            .map_err(|_| "invalid-cost".to_string())
            .and_then(|cost| {
                if cost < 4 || cost > 14 {
                    Err("cost-out-of-range".to_string())
                } else {
                    Ok(cost)
                }
            })
            .and_then(|cost| {
                bcrypt::hash(input.as_bytes(), cost).map_err(|_| "hashing-error".to_string())
            }),
        12.. => SaltString::encode_b64(salt.as_bytes())
            .map_err(|_| "salt-encoding-error".to_string())
            .and_then(|salt| {
                if salt.len() < 1 {
                    Err("salt-too-short".to_string())
                } else {
                    Ok(salt)
                }
            })
            .and_then(|salt| {
                Argon2::default()
                    .hash_password(input.as_bytes(), salt.as_salt())
                    .map(|val| val.to_string())
                    .map_err(|_| "hashing-error".to_string())
            }),
    }
}

fn generate_hash_from_file(
    file_handle: FileHandle,
    algorithm: usize,
    seed: String,
) -> Result<String, String> {
    match File::open(file_handle.path()) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut buffer = [0u8; 8192];
            match algorithm {
                0..=5 => {
                    let mut hasher: Box<dyn DynDigest> = match algorithm {
                        0 => Box::new(Md5::default()),
                        1 => Box::new(Sha1::default()),
                        2 => Box::new(Sha256::default()),
                        3 => Box::new(Sha512::default()),
                        4 => Box::new(Sha3_256::default()),
                        5.. => Box::new(Sha3_512::default()),
                    };
                    loop {
                        let bytes_read = reader.read(&mut buffer);
                        if let Ok(bytes_read) = bytes_read {
                            if bytes_read == 0 {
                                break;
                            }
                            hasher.update(&buffer[..bytes_read]);
                        } else {
                            return Err("error-reading-file".to_string());
                        }
                    }
                    Ok(hex::encode(hasher.finalize()))
                }
                6 => {
                    let mut hasher = crc32fast::Hasher::default();
                    loop {
                        let bytes_read = reader.read(&mut buffer);
                        if let Ok(bytes_read) = bytes_read {
                            if bytes_read == 0 {
                                break;
                            }
                            hasher.update(&buffer[..bytes_read]);
                        } else {
                            return Err("error-reading-file".to_string());
                        }
                    }
                    Ok(hasher.finalize().to_string())
                }
                7 => adler2::adler32(reader)
                    .map_err(|_| "hashing-error".to_string())
                    .and_then(|bytes| Ok(bytes.to_string())),
                8..=9 => seed
                    .parse::<u32>()
                    .map_err(|_| "invalid-seed".to_string())
                    .and_then(|seed| match algorithm {
                        ..=8 => murmur3_32(&mut reader, seed)
                            .map(|val| val.to_string())
                            .map_err(|_| "hashing-error".to_string()),
                        9.. => murmur3_x64_128(&mut reader, seed)
                            .map(|val| val.to_string())
                            .map_err(|_| "hashing-error".to_string()),
                    }),
                10.. => seed
                    .parse::<usize>()
                    .map_err(|_| "invalid-seed".to_string())
                    .and_then(|seed| {
                        let mut hasher = FxHasher::with_seed(seed);
                        loop {
                            let bytes_read = reader.read(&mut buffer);
                            if let Ok(bytes_read) = bytes_read {
                                if bytes_read == 0 {
                                    break;
                                }
                                hasher.write(&buffer[..bytes_read]);
                            } else {
                                return Err("error-reading-file".to_string());
                            }
                        }
                        Ok(hasher.finish().to_string())
                    }),
            }
        }
        Err(_) => Err("cannot-open-file".to_string()),
    }
}

async fn generate_hash(
    input: Option<String>,
    file_handle: Option<FileHandle>,
    algorithm: usize,
    seed: String,
    cost: String,
    salt: String,
) -> Result<String, String> {
    if let Some(file_handle) = file_handle {
        return generate_hash_from_file(file_handle, algorithm, seed);
    } else if let Some(input) = input {
        return generate_hash_from_input(input, algorithm, seed, cost, salt);
    } else {
        return Err("not-supported".to_string());
    }
}

impl UtilityPage for HashGeneratorPage {
    fn get_utility_page(&self) -> Element<'_, Message> {
        let space_s = cosmic::theme::spacing().space_s;

        let header = page_header(HashGeneratorPage::PAGE_NAME);

        let options_header: Element<'_, Message> = widget::text::title4(fl!("options"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let hash_algorithm_option: Element<'_, Message> = widget::settings::item(
            fl!("hash-generator", "hash-algorithm"),
            widget::dropdown(ALGORITHMS.to_vec(), Some(self.algorithm), |selection| {
                Message::HashGeneratorMessage(HashGeneratorMessage::AlgorithmChanged(selection))
            })
            .width(Length::Fixed(120.0)),
        )
        .into();

        let mut column = column::with_capacity(10)
            .spacing(space_s)
            .height(Length::Fill);

        column = column
            .push(header)
            .push(options_header)
            .push(hash_algorithm_option);

        let mut optional_inputs = column::with_capacity(1);

        if (8..=10).contains(&(self.algorithm as i32)) {
            let seed_input: TextInput<'_, Message> = widget::text_input("", &self.seed)
                .id(Id::new(SEED_TEXT_ID))
                .style(text_input_style())
                .helper_text(fl!("hash-generator", "seed"))
                .trailing_icon(
                    row![randomize_button(SEED_TEXT_ID), paste_button(SEED_TEXT_ID)].into(),
                )
                .on_input(|text| {
                    Message::HashGeneratorMessage(HashGeneratorMessage::SeedTextChanged(text))
                })
                .on_submit(|_| Message::HashGeneratorMessage(HashGeneratorMessage::GenerateHash));
            optional_inputs = optional_inputs.push(seed_input);
        }

        if self.algorithm == 11 {
            let cost_input: TextInput<'_, Message> = widget::text_input("", &self.cost)
                .id(Id::new(COST_TEXT_ID))
                .style(text_input_style())
                .helper_text(fl!("hash-generator", "cost"))
                .trailing_icon(paste_button(COST_TEXT_ID).into())
                .on_input(|text| {
                    Message::HashGeneratorMessage(HashGeneratorMessage::CostTextChanged(text))
                })
                .on_submit(|_| Message::HashGeneratorMessage(HashGeneratorMessage::GenerateHash));
            optional_inputs = optional_inputs.push(cost_input);
        }

        if self.algorithm == 12 {
            let salt_input: TextInput<'_, Message> = widget::text_input("", &self.salt)
                .id(Id::new(SALT_TEXT_ID))
                .style(text_input_style())
                .helper_text(fl!("hash-generator", "salt"))
                .trailing_icon(
                    row![randomize_button(SALT_TEXT_ID), paste_button(SALT_TEXT_ID)].into(),
                )
                .on_input(|text| {
                    Message::HashGeneratorMessage(HashGeneratorMessage::SaltTextChanged(text))
                })
                .on_submit(|_| Message::HashGeneratorMessage(HashGeneratorMessage::GenerateHash));
            optional_inputs = optional_inputs.push(salt_input);
        }

        column = column.push(optional_inputs);

        let mut input_header = widget::Row::new();
        input_header =
            input_header
                .push(
                    widget::text::title4(fl!("input"))
                        .width(Length::Fill)
                        .align_x(Alignment::Start),
                )
                .push(widget::button::text(fl!("generate")).on_press(
                    Message::HashGeneratorMessage(HashGeneratorMessage::GenerateHash),
                ));

        if self.algorithm <= 10 {
            input_header = input_header.push(open_file_button());
        }
        input_header = input_header
            .push(paste_button(INPUT_EDITOR_ID))
            .push(clear_button());

        column = column.push(input_header);

        if self.file_handle.is_none() {
            let input_editor: Element<'_, Message> = input_text_editor(
                &self.input_content,
                |action| {
                    Message::HashGeneratorMessage(HashGeneratorMessage::InputEditorAction(action))
                },
                None,
                None::<u32>,
            );
            column = column.push(input_editor);
        } else {
            let file_name = widget::text::title4(self.file_handle.as_ref().unwrap().file_name())
                .height(Length::Fill)
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .align_x(Alignment::Center);
            column = column.push(file_name);
        }

        let output_header: Element<'_, Message> = widget::text::title4(fl!("output"))
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .into();

        let hash_output: TextInput<'_, Message> = widget::text_input("", &self.hash_text)
            .id(Id::new(HASH_TEXT_ID))
            .style(text_input_style())
            .editing(false)
            .trailing_icon(copy_button(HASH_TEXT_ID).into())
            .on_input(|_| {
                Message::HashGeneratorMessage(HashGeneratorMessage::ReadOnlyInput(Id::new(
                    HASH_TEXT_ID,
                )))
            })
            .select_on_focus(true)
            .on_focus(Message::HashGeneratorMessage(
                HashGeneratorMessage::SelectAllTextField(Id::new(HASH_TEXT_ID)),
            ));

        let status_bar = status_bar(&self.status.as_str(), HashGeneratorPage::PAGE_NAME);

        column
            .push(output_header)
            .push(hash_output)
            .push(status_bar)
            .into()
    }

    fn handle_message(
        &mut self,
        message: Message,
    ) -> Task<cosmic::Action<<AppModel as Application>::Message>> {
        match message {
            Message::HashGeneratorMessage(hash_generator_message) => match hash_generator_message {
                HashGeneratorMessage::AlgorithmChanged(selection) => {
                    self.algorithm = selection;
                    self.hash_text = "".to_string();
                    if self.algorithm > 10 {
                        self.file_handle = None;
                    }
                }
                HashGeneratorMessage::SeedTextChanged(text) => {
                    self.seed = text;
                }
                HashGeneratorMessage::CostTextChanged(text) => {
                    self.cost = text;
                }
                HashGeneratorMessage::SaltTextChanged(text) => {
                    self.salt = text;
                }
                HashGeneratorMessage::Randomize(id) => {
                    if id == Id::new(SEED_TEXT_ID) {
                        self.seed = rand::random::<u32>().to_string();
                    } else if id == Id::new(SALT_TEXT_ID) {
                        self.salt = SaltString::generate(&mut OsRng).to_string();
                    }
                }
                HashGeneratorMessage::InputEditorAction(action) => {
                    self.input_content.perform(action);
                }
                HashGeneratorMessage::GenerateHash => {
                    self.status = "generating".to_string();
                    let input = self.input_content.text().clone();
                    let file = self.file_handle.clone();
                    let algorithm = self.algorithm;
                    let seed = self.seed.clone();
                    let cost = self.cost.clone();
                    let salt = self.salt.clone();
                    return Task::future(generate_hash(
                        Some(input),
                        file,
                        algorithm,
                        seed,
                        cost,
                        salt,
                    ))
                    .map(move |result| {
                        cosmic::Action::App(Message::HashGeneratorMessage(
                            HashGeneratorMessage::GenerateHashResult(result),
                        ))
                    });
                }
                HashGeneratorMessage::GenerateHashResult(result) => {
                    match result {
                        Ok(hash) => {
                            self.hash_text = hash;
                            self.status = "ok".to_string();
                        }
                        Err(err) => self.status = err,
                    };
                }
                HashGeneratorMessage::OpenFile => {
                    return Task::perform(
                        async move {
                            let file = AsyncFileDialog::new().set_directory("~").pick_file().await;
                            cosmic::Action::App(Message::HashGeneratorMessage(
                                HashGeneratorMessage::FileSelected(file),
                            ))
                        },
                        |x| x,
                    );
                }
                HashGeneratorMessage::FileSelected(file) => {
                    self.file_handle = file;
                }
                HashGeneratorMessage::ClearContent => {
                    self.file_handle = None;
                    self.input_content.perform(text_editor::Action::SelectAll);
                    self.input_content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Backspace));
                }
                HashGeneratorMessage::SelectAllTextField(id) => {
                    return text_input::select_all(id);
                }
                HashGeneratorMessage::CopyText(id) => {
                    if id == Id::new(HASH_TEXT_ID) {
                        return clipboard::write(self.hash_text.clone());
                    }
                }
                HashGeneratorMessage::PasteText(id) => {
                    return clipboard::read().map(move |optional_data| match optional_data {
                        Some(data) => cosmic::Action::App(Message::HashGeneratorMessage(
                            HashGeneratorMessage::ReplaceText(id.clone(), data),
                        )),
                        None => cosmic::Action::App(Message::HashGeneratorMessage(
                            HashGeneratorMessage::NoOp,
                        )),
                    });
                }
                HashGeneratorMessage::ReplaceText(id, text) => {
                    if id == Id::new(INPUT_EDITOR_ID) {
                        self.input_content.perform(text_editor::Action::SelectAll);
                        self.input_content.perform(text_editor::Action::Edit(
                            text_editor::Edit::Paste(Arc::new(text)),
                        ));
                    } else if id == Id::new(SEED_TEXT_ID) {
                        self.seed = text;
                    } else if id == Id::new(COST_TEXT_ID) {
                        self.cost = text;
                    } else if id == Id::new(SALT_TEXT_ID) {
                        self.salt = text;
                    }
                }
                HashGeneratorMessage::ReadOnlyInput(id) => {
                    return text_input::move_cursor_to_end(id);
                }
                HashGeneratorMessage::NoOp => {}
            },
            _ => {
                println!("Wrong message type");
            }
        };
        Task::none()
    }
}

impl HashGeneratorPage {
    const PAGE_NAME: &str = "hash-generator";
}
