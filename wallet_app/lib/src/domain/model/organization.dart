import 'package:equatable/equatable.dart';

import 'app_image_data.dart';
import 'localized_text.dart';

class Organization extends Equatable {
  final String id;
  final LocalizedText legalName;
  final LocalizedText displayName;
  final LocalizedText? category;
  final LocalizedText? description;
  final AppImageData logo;
  final String? webUrl;
  final String? privacyPolicyUrl;
  final String? countryCode;
  final LocalizedText? city;
  final LocalizedText? department;
  final String? kvk;

  const Organization({
    required this.id,
    required this.legalName,
    required this.displayName,
    required this.category,
    required this.description,
    required this.logo,
    this.webUrl,
    this.privacyPolicyUrl,
    this.countryCode,
    this.city,
    this.department,
    this.kvk,
  });

  @override
  List<Object?> get props => [
        id,
        legalName,
        displayName,
        category,
        description,
        logo,
        webUrl,
        privacyPolicyUrl,
        city,
        countryCode,
        department,
        kvk,
      ];

  Organization copyWith({LocalizedText? displayName}) {
    return Organization(
      id: id,
      legalName: legalName,
      displayName: displayName ?? this.displayName,
      category: category,
      description: description,
      logo: logo,
      webUrl: webUrl,
      privacyPolicyUrl: privacyPolicyUrl,
      countryCode: countryCode,
      city: city,
      department: department,
      kvk: kvk,
    );
  }
}
