pub mod unix_time_converter_page;

use {
    crate::{Message, app::AppModel},
    cosmic::{Application, Element, app::Task},
};

pub(crate) trait UtilityPage {
    fn get_utility_page(&self) -> Element<'_, Message>;

    fn handle_message(&mut self, message: Message) -> Task<<AppModel as Application>::Message>;
}
