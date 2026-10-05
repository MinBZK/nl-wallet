import 'package:freezed_annotation/freezed_annotation.dart';

import 'app_image_data.dart';
import 'attribute/converter/localized_text_converter.dart';
import 'converter/app_image_data_converter.dart';
import 'localized_text.dart';

part 'organization.freezed.dart';
part 'organization.g.dart';

@freezed
abstract class Organization with _$Organization {
  const factory Organization({
    required String id,
    required String legalName,
    required String displayName,
    @LocalizedTextConverter() LocalizedText? type,
    @LocalizedTextConverter() LocalizedText? description,
    @AppImageDataConverter() AppImageData? logo,
    String? webUri,
    String? supportUri,
    String? privacyPolicyUri,
    required String countryCode,
    required String organizationId,
  }) = _Organization;

  factory Organization.fromJson(Map<String, dynamic> json) => _$OrganizationFromJson(json);
}
