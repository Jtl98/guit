use crate::diff::DiffKey;

pub enum Action {
    Pull,
    Refresh,
    AddOrRestore(DiffKey),
}
