import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../../util/extension/card_display_metadata_list_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../attribute/attribute.dart';
import '../organization.dart';
import 'card_config.dart';
import 'card_front.dart';
import 'metadata/card_display_metadata.dart';

part 'wallet_card.g.dart';

@JsonSerializable(explicitToJson: true)
class WalletCard extends Equatable {
  final String id;
  final String docType;
  final Organization issuer;
  final CardFront? front; // Legacy UI attributes, only used for mock & test
  final List<CardDisplayMetadata> metadata;
  final List<DataAttribute> attributes;
  final CardConfig config;

  LocalizedText get title => metadata.name ?? front?.title ?? ''.untranslated;

  LocalizedText get description => metadata.description ?? front?.subtitle ?? ''.untranslated;

  const WalletCard({
    required this.id,
    required this.docType,
    required this.issuer,
    this.front,
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
        front,
        attributes,
        metadata,
        config,
      ];
}
