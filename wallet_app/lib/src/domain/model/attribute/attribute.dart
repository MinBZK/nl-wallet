import 'package:equatable/equatable.dart';

import '../localized_text.dart';
import 'attribute_value.dart';

export '../../../util/extension/localized_text_extension.dart';
export '../localized_text.dart';
export 'attribute_value.dart';

/// The base class to represent a card's attribute inside the app.
abstract class Attribute extends Equatable {
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

typedef AttributeKey = String;
