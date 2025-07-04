import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../../util/extension/card_display_metadata_list_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/mapper/card/summary_mapper.dart';
import '../attribute/attribute.dart';
import '../organization.dart';
import 'card_config.dart';
import 'metadata/card_display_metadata.dart';

part 'wallet_card.g.dart';

@JsonSerializable(explicitToJson: true)
class WalletCard extends Equatable {
  /// ID of the card, null when the card is not persisted in the database
  final String? id;

  /// Type of document
  final String docType;

  /// Organization that issued this card
  final Organization issuer;

  /// Card display metadata for UI rendering
  final List<CardDisplayMetadata> metadata;

  /// Data attributes stored in the card
  final List<DataAttribute> attributes;

  /// Configuration settings for card behavior/appearance (used in mock builds)
  final CardConfig config;

  /// Indicates whether the card is persisted in the database.
  bool get isPersisted => id != null;

  LocalizedText get title => metadata.name ?? ''.untranslated;

  LocalizedText get description => metadata.description ?? ''.untranslated;

  LocalizedText get summary => CardSummaryMapper().map(this);

  const WalletCard({
    required this.id,
    required this.docType,
    required this.issuer,
    required this.attributes,
    this.metadata = const [],
    this.config = const CardConfig(),
  });

  factory WalletCard.fromJson(Map<String, dynamic> json) => _$WalletCardFromJson(json);

  Map<String, dynamic> toJson() => _$WalletCardToJson(this);

  @override
  List<Object?> get props => [
        id,
        docType,
        issuer,
        attributes,
        metadata,
        config,
      ];

  WalletCard copyWith({
    String? Function()? id,
    String? docType,
    Organization? issuer,
    List<DataAttribute>? attributes,
    List<CardDisplayMetadata>? metadata,
    CardConfig? config,
  }) {
    return WalletCard(
      id: id != null ? id() : this.id,
      docType: docType ?? this.docType,
      issuer: issuer ?? this.issuer,
      attributes: attributes ?? this.attributes,
      metadata: metadata ?? this.metadata,
      config: config ?? this.config,
    );
  }
}
