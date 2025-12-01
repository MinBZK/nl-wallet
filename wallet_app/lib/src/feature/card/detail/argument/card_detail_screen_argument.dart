import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../../domain/model/attribute/converter/localized_text_converter.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../domain/model/localized_text.dart';

part 'card_detail_screen_argument.freezed.dart';
part 'card_detail_screen_argument.g.dart';

@Freezed(copyWith: false)
abstract class CardDetailScreenArgument with _$CardDetailScreenArgument {
  const factory CardDetailScreenArgument({
    WalletCard? card,
    required String cardId,
    @LocalizedTextConverter() required LocalizedText cardTitle,
  }) = _CardDetailScreenArgument;

  const CardDetailScreenArgument._();

  factory CardDetailScreenArgument.forCard(WalletCard card) {
    assert(
      card.isPersisted,
      'Card details screen can only be opened for persisted cards, providing id-less cards will render an error screen.',
    );
    return CardDetailScreenArgument(
      card: card,
      cardId: card.attestationId ?? '',
      cardTitle: card.title,
    );
  }

  factory CardDetailScreenArgument.fromJson(Map<String, dynamic> json) => _$CardDetailScreenArgumentFromJson(json);
}
