// SPDX-License-Identifier: GPL-3.0

use {
    crate::{
        ContextPage, Message,
        config::Config,
        context_drawer_pages::{ContextDrawerPage, about_page::AboutPage},
        fl,
        utility_pages::{
            UtilityPage, data_converter_formatter_page::DataConverterFormatterPage,
            unix_time_converter_page::UnixTimeConverterPage,
        },
    },
    cosmic::{
        ApplicationExt, Apply, Element, Task,
        app::context_drawer,
        cosmic_config::{self, CosmicConfigEntry},
        iced::{
            Length, Padding, Subscription,
            alignment::{Horizontal, Vertical},
        },
        widget::{self, icon, menu, nav_bar},
    },
    std::{collections::HashMap, hash::Hash},
};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
    utility_pages: HashMap<Page, Box<dyn UtilityPage>>,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "io.github.avomar.dev-toolbox";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("unix-time-converter"))
            .data::<Page>(Page::UnixTimeConverter)
            .icon(icon::from_name("accessories-clock-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("data-converter-formatter"))
            .data::<Page>(Page::DataConverterFormatter)
            .icon(icon::from_name("x-office-document-symbolic"));

        let mut utility_pages = HashMap::<Page, Box<dyn UtilityPage>>::new();
        utility_pages.insert(
            Page::UnixTimeConverter,
            Box::new(UnixTimeConverterPage::default()),
        );
        utility_pages.insert(
            Page::DataConverterFormatter,
            Box::new(DataConverterFormatterPage::default()),
        );

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            utility_pages,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                AboutPage::get_context_drawer_page(),
                Message::ToggleContextPage(ContextPage::About),
            ),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let content = self
            .utility_pages
            .get(self.nav.active_data::<Page>().unwrap())
            .unwrap()
            .get_utility_page();

        widget::container(content)
            .width(600)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .padding(Padding::new(0.0).bottom(8.0))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.set_show_context(!self.core.window.show_context);
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::UnixTimeConverterMessage(_) => {
                return self
                    .utility_pages
                    .get_mut(&Page::UnixTimeConverter)
                    .unwrap()
                    .handle_message(message);
            }
            Message::DataConverterFormatterMessage(_) => {
                return self
                    .utility_pages
                    .get_mut(&Page::DataConverterFormatter)
                    .unwrap()
                    .handle_message(message);
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_title()
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

/// The page to display in the application.
#[derive(Eq, Hash, PartialEq)]
pub enum Page {
    UnixTimeConverter,
    DataConverterFormatter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
