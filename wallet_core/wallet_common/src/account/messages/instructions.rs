use crate::account::signed::SignedDouble;

use super::auth::WalletCertificate;

#[allow(unused)]
struct Instruction<T: IsInstruction> {
    instruction: SignedDouble<T>,
    certificate: WalletCertificate,
}

trait IsInstruction {}

struct CheckPin;

impl IsInstruction for CheckPin {}
