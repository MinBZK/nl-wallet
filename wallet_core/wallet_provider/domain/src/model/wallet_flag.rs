#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumCount, strum::EnumIter, strum::EnumString,
)]
pub enum WalletFlag {
    SolutionRevoked,
}
