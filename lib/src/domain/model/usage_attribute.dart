import 'package:equatable/equatable.dart';

class UsageAttribute extends Equatable {
  final String value;
  final String status;

  const UsageAttribute({
    required this.value,
    required this.status,
  });

  @override
  List<Object?> get props => [value, status];
}
