pub struct NavigationHistory {
    history: Vec<usize>,
    cursor: usize,
}

#[derive(Debug, Clone)]
pub enum NavigationChange {
    Back,
    Forward,
}

impl NavigationHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            cursor: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            history: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }

    pub fn push_history(&mut self, page: usize) -> Vec<usize> {
        let mut removed = Vec::new();

        if !self.history.is_empty() {
            let cut = self.cursor + 1;

            if cut < self.history.len() {
                removed = self.history.split_off(cut);
            }
        }

        self.history.push(page);
        self.cursor = self.history.len() - 1;

        removed
    }

    pub fn contains(&self, page: usize) -> bool {
        self.history.contains(&page)
    }

    pub fn nav_back(&mut self) -> Option<usize> {
        if self.cursor > 0 {
            self.cursor -= 1;
            return Some(self.history[self.cursor]);
        }

        None
    }

    pub fn nav_forward(&mut self) -> Option<usize> {
        if self.cursor + 1 < self.history.len() {
            self.cursor += 1;
            return Some(self.history[self.cursor]);
        }

        None
    }
}
