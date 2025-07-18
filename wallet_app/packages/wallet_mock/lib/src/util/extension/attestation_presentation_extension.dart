import 'package:wallet_core/core.dart';

extension AttestationPresentationExtension on AttestationPresentation {
  AttestationPresentation fixed() {
    return AttestationPresentation(
      identity: AttestationIdentity.fixed(id: attestationType),
      attestationType: attestationType,
      displayMetadata: displayMetadata,
      issuer: issuer,
      attributes: attributes,
    );
  }
}
