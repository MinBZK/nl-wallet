import 'package:equatable/equatable.dart';

class Verifier extends Equatable {
  final String name;
  final String shortName;
  final String description;
  final String? logoUrl;

  const Verifier({
    required this.name,
    required this.shortName,
    required this.description,
    this.logoUrl,
  });

  @override
  List<Object?> get props => [name, shortName, description, logoUrl];
}
