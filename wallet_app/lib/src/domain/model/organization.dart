import 'package:freezed_annotation/freezed_annotation.dart';

import 'attribute/converter/localized_text_converter.dart';
import 'localized_text.dart';
import 'person_type.dart';

part 'organization.freezed.dart';
part 'organization.g.dart';

@freezed
abstract class Organization with _$Organization {
  const factory Organization({
    required String id,
    required String legalName,
    required String displayName,
    bool? publicBody,
    required PersonType personType,
    @LocalizedTextConverter() LocalizedText? description,
    String? webUri,
    required String supportUri,
    String? privacyPolicyUri,
    required String countryCode,
    required String organizationId,
  }) = _Organization;

  factory Organization.fromJson(Map<String, dynamic> json) => _$OrganizationFromJson(json);
}
