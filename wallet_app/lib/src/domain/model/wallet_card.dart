import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import 'attribute/data_attribute.dart';
import 'card_config.dart';
import 'card_front.dart';

part 'wallet_card.g.dart';

@JsonSerializable(explicitToJson: true)
class WalletCard extends Equatable {
  final String id;
  final String docType;
  final CardFront front;
  final List<DataAttribute> attributes;
  final CardConfig config;

  const WalletCard({
    required this.id,
    required this.docType,
    required this.front,
    required this.attributes,
    this.config = const CardConfig(),
  });

  factory WalletCard.fromJson(Map<String, dynamic> json) => _$WalletCardFromJson(json);

  Map<String, dynamic> toJson() => _$WalletCardToJson(this);

  @override
  List<Object> get props => [id, front, attributes, config];
}
