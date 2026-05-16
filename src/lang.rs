#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Chinese,
    English,
}

impl Language {
    pub fn toggle(&mut self) {
        *self = match self {
            Language::Chinese => Language::English,
            Language::English => Language::Chinese,
        };
    }

    pub fn is_chinese(self) -> bool {
        matches!(self, Language::Chinese)
    }
}
