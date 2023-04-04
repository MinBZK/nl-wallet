import 'package:equatable/equatable.dart';

class Organization extends Equatable {
  final String id;
  final String name;
  final String shortName;
  final String description;
  final String logoUrl;

  const Organization({
    required this.id,
    required this.name,
    required this.shortName,
    required this.description,
    required this.logoUrl,
  });

  @override
  List<Object?> get props => [id, name, shortName, description, logoUrl];
}
