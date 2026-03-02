import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:wallet_core/core.dart';

import '../core_error.dart';
import '../core_error_mapper.dart';

part 'core_error_data.freezed.dart';
part 'core_error_data.g.dart';

/// A unified data container for various [CoreError] types.
///
/// This class safely parses different potential error payloads. Since an
/// error typically only contains a subset of these fields, it is the
/// responsibility of the [CoreErrorMapper] to extract the relevant data
/// for a specific error type.
@freezed
abstract class CoreErrorData with _$CoreErrorData {
  factory CoreErrorData({
    @JsonKey(name: 'revocation_data') RevocationData? revocationData,
    @JsonKey(name: 'redirect_error', unknownEnumValue: RedirectError.unknown) RedirectError? redirectError,
    @JsonKey(name: 'session_type', unknownEnumValue: SessionType.unknown) SessionType? sessionType,
    @JsonKey(name: 'can_retry') bool? canRetry,
    @JsonKey(name: 'organization_name') Map<String, dynamic>? organizationName,
  }) = _CoreErrorData;

  const CoreErrorData._();

  factory CoreErrorData.fromJson(Map<String, dynamic> json) => _$CoreErrorDataFromJson(json);

  List<LocalizedString>? get mappedOrganizationName {
    final data = organizationName;
    if (data == null || data.isEmpty) return null;

    return [for (final entry in data.entries) LocalizedString(language: entry.key, value: entry.value)];
  }
}
