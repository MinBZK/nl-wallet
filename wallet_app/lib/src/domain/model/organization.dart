import 'package:equatable/equatable.dart';

import 'localized_text.dart';

class Organization extends Equatable {
  final String id;
  final LocalizedText legalName;
  final LocalizedText displayName;
  final LocalizedText? type;
  final LocalizedText? description;
  final String logoUrl;
  final String? webUrl;
  final LocalizedText? country;
  final LocalizedText? city;
  final LocalizedText? department;

  const Organization({
    required this.id,
    required this.legalName,
    required this.displayName,
    required this.type,
    required this.description,
    required this.logoUrl,
    this.webUrl,
    this.country,
    this.city,
    this.department,
  });

  @override
  List<Object?> get props =>
      [id, legalName, displayName, type, description, logoUrl, webUrl, city, country, department];
}
