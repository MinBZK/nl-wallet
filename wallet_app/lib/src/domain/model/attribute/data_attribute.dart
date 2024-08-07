// ignore_for_file: tighten_type_of_initializing_formals

import 'package:json_annotation/json_annotation.dart';

import 'attribute.dart';
import 'converter/attribute_value_converter.dart';
import 'converter/localized_string_converter.dart';

part 'data_attribute.g.dart';

/// A [DataAttribute] represents an attribute that is available in the user's wallet.
/// As such it will always contain a valid [AttributeValue].
@JsonSerializable(converters: [AttributeValueConverter(), LocalizedStringConverter()])
class DataAttribute extends Attribute {
  @override
  AttributeValue get value => super.value!;

  final String sourceCardDocType;

  const DataAttribute({
    required super.key,
    required super.label,
    required super.value,
    required this.sourceCardDocType,
  }) : assert(value != null, 'Value of DataAttribute should never null');

  DataAttribute.untranslated({
    required super.key,
    required String label,
    required super.value,
    required this.sourceCardDocType,
  })  : assert(value != null, 'Value of DataAttribute should never null'),
        super(label: {'': label});

  factory DataAttribute.fromJson(Map<String, dynamic> json) => _$DataAttributeFromJson(json);

  Map<String, dynamic> toJson() => _$DataAttributeToJson(this);

  @override
  List<Object?> get props => [key, label, value, sourceCardDocType];
}
