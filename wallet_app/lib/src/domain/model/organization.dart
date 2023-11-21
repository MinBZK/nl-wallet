import 'package:equatable/equatable.dart';

import 'app_image_data.dart';
import 'localized_text.dart';

class Organization extends Equatable {
  final String id;
  final LocalizedText legalName;
  final LocalizedText displayName;
  final LocalizedText? type;
  final LocalizedText? description;
  final AppImageData logo;
  final String? webUrl;
  final String? countryCode;
  final LocalizedText? city;
  final LocalizedText? department;
  final String? kvk;

  const Organization({
    required this.id,
    required this.legalName,
    required this.displayName,
    required this.type,
    required this.description,
    required this.logo,
    this.webUrl,
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
        type,
        description,
        logo,
        webUrl,
        city,
        countryCode,
        department,
        kvk,
      ];
}
