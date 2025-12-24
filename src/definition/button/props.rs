pub struct UiButtonProperties {
    hier: ButtonHierarchy,
    size: ButtonSize,
}

impl UiButtonProperties {
    pub(crate) fn get_hier(&self) -> &ButtonHierarchy {
        &self.hier
    }

    pub(crate) fn get_size(&self) -> &ButtonSize {
        &self.size
    }

    pub(crate) fn set_hier(&mut self, hier: ButtonHierarchy) {
        self.hier = hier;
    }

    pub(crate) fn set_size(&mut self, size: ButtonSize) {
        self.size = size
    }
}

impl Default for UiButtonProperties {
    fn default() -> Self {
        Self {
            hier: ButtonHierarchy::Primary,
            size: ButtonSize::SM,
        }
    }
}

#[derive(Clone)]
pub enum ButtonHierarchy {
    Primary,
    Secondary,
    // Tertiary,
}

#[derive(Clone)]
pub enum ButtonSize {
    SM,
    MD,
    LG,
}
