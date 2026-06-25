import 'package:freezed_annotation/freezed_annotation.dart';

import '../card/format/attestation_format.dart';

part 'pid_attestation.freezed.dart';

@freezed
abstract class PidAttestation with _$PidAttestation {
  const factory PidAttestation({
    required String attestationType,
    required AttestationFormat format,
  }) = _PidAttestation;

  const PidAttestation._();
}
