// Derived from Just Enough Items (https://github.com/mezz/JustEnoughItems),
// Gui/src/main/java/mezz/jei/gui/recipes/NavigationHistory.java and its test.
// Copyright (c) 2014-2015 mezz. MIT licence: LICENSES/JustEnoughItems-MIT.txt.

//! Back and forward through the pages the player opened.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationHistory<T> {
    backward: Vec<T>,
    forward: Vec<T>,
}

impl<T> Default for NavigationHistory<T> {
    fn default() -> Self {
        Self {
            backward: Vec::new(),
            forward: Vec::new(),
        }
    }
}

impl<T> NavigationHistory<T> {
    pub fn record(&mut self, value: T) {
        self.backward.push(value);
        self.forward.clear();
    }

    pub fn go_back(&mut self, current: T) -> Option<T> {
        let previous = self.backward.pop()?;
        self.forward.push(current);
        Some(previous)
    }

    pub fn go_forward(&mut self, current: T) -> Option<T> {
        let next = self.forward.pop()?;
        self.backward.push(current);
        Some(next)
    }

    pub fn can_go_back(&self) -> bool {
        !self.backward.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    pub fn clear(&mut self) {
        self.backward.clear();
        self.forward.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigates_backward_and_forward() {
        let mut history = NavigationHistory::default();
        history.record("first");
        history.record("second");
        let current = history.go_back("third").unwrap();
        assert_eq!(current, "second");
        let current = history.go_back(current).unwrap();
        assert_eq!(current, "first");
        let current = history.go_forward(current).unwrap();
        assert_eq!(current, "second");
        let current = history.go_forward(current).unwrap();
        assert_eq!(current, "third");
    }

    #[test]
    fn recording_after_going_back_clears_forward_history() {
        let mut history = NavigationHistory::default();
        history.record("first");
        history.record("second");
        let current = history.go_back("third").unwrap();
        history.record(current);
        assert!(history.go_forward("new branch").is_none());
        assert_eq!(history.go_back("new branch"), Some("second"));
    }

    #[test]
    fn clear_removes_backward_and_forward_history() {
        let mut history = NavigationHistory::default();
        history.record("first");
        let current = history.go_back("second").unwrap();
        history.clear();
        assert!(history.go_back(current).is_none());
        assert!(history.go_forward(current).is_none());
    }
}
