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
    @LocalizedTextConverter() required LocalizedText legalName,
    @LocalizedTextConverter() required LocalizedText displayName,
    @LocalizedTextConverter() required LocalizedText? category,
    @LocalizedTextConverter() required LocalizedText? description,
    @AppImageDataConverter() required AppImageData logo,
    String? webUrl,
    String? privacyPolicyUrl,
    String? countryCode,
    @LocalizedTextConverter() LocalizedText? city,
    @LocalizedTextConverter() LocalizedText? department,
    String? kvk,
  }) = _Organization;

  factory Organization.fromJson(Map<String, dynamic> json) => _$OrganizationFromJson(json);
}
