import 'package:equatable/equatable.dart';

class DataAttribute extends Equatable {
  final DataAttributeType type;
  final String label;
  final String? value;

  const DataAttribute({
    required this.type,
    required this.label,
    required this.value,
  });

  @override
  List<Object?> get props => [type, label, value];
}

enum DataAttributeType { image, text }
