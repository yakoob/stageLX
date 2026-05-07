#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeStrategy {
    #[default]
    Htp,
    Ltp,
}
