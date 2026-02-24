pub enum Action {
    Pull,
    Refresh,
    RefreshStaged,
    Diff(String),
    DiffStaged(String),
}
