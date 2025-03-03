import 'package:equatable/equatable.dart';
import 'package:json_annotation/json_annotation.dart';

import '../../../../domain/model/attribute/converter/localized_string_converter.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/localized_text.dart';

part 'card_detail_screen_argument.g.dart';

@JsonSerializable(converters: [LocalizedStringConverter()], explicitToJson: true)
class CardDetailScreenArgument extends Equatable {
  final WalletCard? card;
  final String cardId;
  final LocalizedText cardTitle;

  const CardDetailScreenArgument({this.card, required this.cardId, required this.cardTitle});

  factory CardDetailScreenArgument.forCard(WalletCard card) => CardDetailScreenArgument(
        card: card,
        cardId: card.id,
        cardTitle: card.title,
      );

  factory CardDetailScreenArgument.fromJson(Map<String, dynamic> json) => _$CardDetailScreenArgumentFromJson(json);

  Map<String, dynamic> toJson() => _$CardDetailScreenArgumentToJson(this);

  @override
  List<Object?> get props => [card, cardId, cardTitle];
}
