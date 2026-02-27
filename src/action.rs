use crate::common::DiffKey;

pub enum Action {
    Pull,
    Refresh,
    AddOrRestore(DiffKey),
}
