use super::{auth::WalletCertificate, signed::SignedDouble};

#[allow(unused)]
struct Instruction<T: IsInstruction> {
    instruction: SignedDouble<T>,
    certificate: WalletCertificate,
}

trait IsInstruction {}

struct CheckPin;

impl IsInstruction for CheckPin {}
