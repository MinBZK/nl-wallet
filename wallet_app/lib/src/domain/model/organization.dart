import 'package:equatable/equatable.dart';

class Organization extends Equatable {
  final String id;
  final String name;
  final String shortName;
  final String category;
  final String description;
  final String logoUrl;
  final String? webUrl;
  final String? location;
  final String? department;

  const Organization({
    required this.id,
    required this.name,
    required this.shortName,
    required this.category,
    required this.description,
    required this.logoUrl,
    this.webUrl,
    this.location,
    this.department,
  });

  @override
  List<Object?> get props => [id, name, shortName, category, description, logoUrl, webUrl, location, department];
}
