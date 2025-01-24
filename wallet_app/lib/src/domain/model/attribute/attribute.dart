import 'package:equatable/equatable.dart';
import 'package:flutter/widgets.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../../feature/common/widget/attribute/ui_attribute_row.dart';
import '../localized_text.dart';
import 'attribute_value.dart';
import 'converter/attribute_value_converter.dart';
import 'converter/localized_string_converter.dart';

export '../../../util/extension/localized_text_extension.dart';
export '../localized_text.dart';
export 'attribute_value.dart';

part 'attribute.g.dart';

typedef AttributeKey = String;

/// Sealed class for all attribute implementations, can be
/// rendered to the screen using the [AttributeRow] widget.
sealed class Attribute extends Equatable {
  /// Key that uniquely identifies the attribute (within a card)
  final AttributeKey key;

  /// The [Attribute]s label, often shown above the actual value to indicate what the value refers to
  final LocalizedText label;

  /// The value of this [Attribute] nullable because the [value] might not be available in the user's wallet
  final AttributeValue? value;

  const Attribute({
    required this.key,
    required this.label,
    this.value,
  });
}

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
    required AttributeValue super.value,
    required this.sourceCardDocType,
  });

  DataAttribute.untranslated({
    required super.key,
    required String label,
    required AttributeValue super.value,
    required this.sourceCardDocType,
  }) : super(label: {'': label});

  factory DataAttribute.fromJson(Map<String, dynamic> json) => _$DataAttributeFromJson(json);

  Map<String, dynamic> toJson() => _$DataAttributeToJson(this);

  @override
  List<Object?> get props => [key, label, value, sourceCardDocType];
}

/// The sole purpose of a [UiAttribute] is to be rendered to the screen, it should not be used for any (business) logic.
/// A [UiAttribute] is contains an optional [icon] that relates to the data it represents. Note that the
/// alignment of the visualization changes based on the availability of [icon], see [UiAttributeRow].
class UiAttribute extends Attribute {
  @override
  AttributeValue get value => super.value!;

  final IconData? icon;

  const UiAttribute({
    required AttributeValue super.value,
    this.icon,
    super.key = '',
    required super.label,
  });

  UiAttribute.untranslated({
    required super.value,
    required this.icon,
    super.key = '',
    required String label,
  }) : super(label: {'': label});

  @override
  String get key => throw UnsupportedError('UiAttributes should only be used to render data to the screen');

  @override
  List<Object?> get props => [value, icon, label];
}

/// A [MissingAttribute] is used to represent an attribute that was requested by a relying party, but is not (currently)
/// available is the user's wallet. As such it will never contain an [AttributeValue].
class MissingAttribute extends Attribute {
  const MissingAttribute({super.key = '', required super.label});

  MissingAttribute.untranslated({required super.key, required String label}) : super(label: {'': label});

  @override
  List<Object?> get props => [key, label];
}

/// This is conceptually a slight deviation of the original [MissingAttribute] that is only used for Mock builds,
/// it represents an attribute which the relying party requests from the user, but at a stage where we haven't yet
/// checked to see if it's part of the users wallet. Therefor marking it as "Missing" would be invalid.
typedef MockRequestedAttribute = MissingAttribute;
