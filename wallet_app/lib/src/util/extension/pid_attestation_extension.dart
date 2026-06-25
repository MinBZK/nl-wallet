import '../../domain/model/card/wallet_card.dart';
import '../../domain/model/pid/pid_attestation.dart';

extension PidAttestationExtension on PidAttestation {
  bool matches(WalletCard card) => attestationType == card.attestationType && card.format == format;
}
