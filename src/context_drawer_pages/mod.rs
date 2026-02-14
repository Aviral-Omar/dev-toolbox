pub(crate) mod about_page;

use {crate::Message, cosmic::Element};

pub(crate) trait ContextDrawerPage {
    fn get_context_drawer_page() -> Element<'static, Message>;
}
