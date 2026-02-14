use {
    crate::{APP_ICON, Message, REPOSITORY, context_drawer_pages::ContextDrawerPage, fl},
    cosmic::{
        Element,
        widget::{
            self,
            about::{self, About},
        },
    },
    std::sync::OnceLock,
};

static ABOUT_DATA: OnceLock<About> = OnceLock::new();

pub(crate) struct AboutPage {}

impl ContextDrawerPage for AboutPage {
    fn get_context_drawer_page() -> Element<'static, Message> {
        let about = ABOUT_DATA.get_or_init(|| {
            About::default()
                .name(fl!("app-title"))
                .icon(widget::icon::from_svg_bytes(APP_ICON))
                .version(env!("CARGO_PKG_VERSION"))
                .links([(fl!("repository"), REPOSITORY)])
                .license(env!("CARGO_PKG_LICENSE"))
        });

        about::about(about, |url| Message::LaunchUrl(url.to_string()))
    }
}
