import 'package:equatable/equatable.dart';

class TrustProvider extends Equatable {
  final String name;
  final String logoUrl;

  const TrustProvider({
    required this.name,
    required this.logoUrl,
  });

  @override
  List<Object?> get props => [name, logoUrl];
}
