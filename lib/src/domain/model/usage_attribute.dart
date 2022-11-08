import 'package:equatable/equatable.dart';

class UsageAttribute extends Equatable {
  final String value;
  final UsageStatus status;
  final DateTime dateTime;

  const UsageAttribute({
    required this.value,
    required this.status,
    required this.dateTime,
  });

  @override
  List<Object?> get props => [value, status];
}

enum UsageStatus { success, rejected, failed }
