use iced::{
    advanced::widget::tree::{self, Tag},
    widget::text_input,
};

pub enum FocusDetector {
    ClickBased,
    TextInput,
    Custom(Box<dyn Fn(&tree::Tree) -> bool>),
}

impl Default for FocusDetector {
    fn default() -> Self {
        Self::TextInput
    }
}

impl<'a, Message, Theme, Renderer> super::MultiBorder<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    pub fn focus_detector(mut self, detector: FocusDetector) -> Self {
        self.focus_detector = detector;
        self
    }

    pub fn resolve_focus(&self, tree: &tree::Tree, is_click_inside: Option<bool>) -> bool {
        match &self.focus_detector {
            FocusDetector::ClickBased => is_click_inside.unwrap_or(false),
            FocusDetector::TextInput => Self::detect_text_input_focus(&tree.children[0]),
            FocusDetector::Custom(f) => f(&tree.children[0]),
        }
    }

    fn detect_text_input_focus(child_tree: &tree::Tree) -> bool {
        let tag =
            Tag::of::<text_input::State<<Renderer as iced::advanced::text::Renderer>::Paragraph>>();

        if child_tree.tag == tag {
            return child_tree.state.downcast_ref::<text_input::State<
                <Renderer as iced::advanced::text::Renderer>::Paragraph
            >>().is_focused();
        }

        child_tree
            .children
            .iter()
            .any(|c| Self::detect_text_input_focus(c))
    }
}
