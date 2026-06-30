#[derive(Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
}