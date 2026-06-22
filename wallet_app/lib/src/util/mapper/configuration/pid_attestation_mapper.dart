import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/pid/pid_attestation.dart';
import '../mapper.dart';

class PidAttestationMapper extends Mapper<core.PidAttestation, PidAttestation> {
  PidAttestationMapper();

  @override
  PidAttestation map(core.PidAttestation input) {
    return switch (input.format) {
      core.Format.MsoMdoc => PidAttestation(attestationType: input.attestationType, format: .mdoc),
      core.Format.SdJwt => PidAttestation(attestationType: input.attestationType, format: .sdJwt),
    };
  }
}
