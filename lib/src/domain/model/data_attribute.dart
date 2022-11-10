import 'package:equatable/equatable.dart';

class DataAttribute extends Equatable {
  final String type;
  final String? value;

  const DataAttribute({
    required this.type,
    required this.value,
  });

  @override
  List<Object?> get props => [type, value];
}
