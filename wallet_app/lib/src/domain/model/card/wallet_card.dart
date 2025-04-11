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
  final String id;
  final String docType;
  final Organization issuer;
  final List<CardDisplayMetadata> metadata;
  final List<DataAttribute> attributes;
  final CardConfig config;

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
}
