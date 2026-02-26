pub enum Action {
    Pull,
    RefreshUnstaged,
    RefreshStaged,
    DiffUnstaged(String),
    DiffStaged(String),
}
