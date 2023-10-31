import 'attribute.dart';

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
