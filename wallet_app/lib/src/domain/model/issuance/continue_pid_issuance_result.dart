import 'package:freezed_annotation/freezed_annotation.dart';

import '../attribute/attribute.dart';

part 'continue_pid_issuance_result.freezed.dart';

/// The outcome of continuing a PID issuance after the user authenticated.
@freezed
sealed class ContinuePidIssuanceResult with _$ContinuePidIssuanceResult {
  /// Preview of the attributes that will be added if the PID is accepted.
  const factory ContinuePidIssuanceResult.previews(
    List<Attribute> attributes,
  ) = ContinuePidIssuancePreviews;

  /// The issuer does not provide previews, so the PID can only be shown once it is issued.
  const factory ContinuePidIssuanceResult.noPreviews() = ContinuePidIssuanceNoPreviews;
}
