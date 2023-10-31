import 'package:flutter/material.dart';

import 'attribute.dart';

/// The sole purpose of a [UiAttribute] is to render data to the screen, it should not be used for any (business) logic.
/// A [UiAttribute] is enriched with an [icon] that relates to the data it contains.
class UiAttribute extends Attribute {
  @override
  AttributeValue get value => super.value!;

  final IconData icon;

  const UiAttribute({
    required super.value,
    required this.icon,
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
  List<Object?> get props => [value, icon, key, label];
}
