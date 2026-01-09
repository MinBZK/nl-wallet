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

  /// Will load card details for the given [WalletCard]. This allow the [CardDetailScreen] to instantly render the
  /// [WalletCard] at the top of the screen, allowing for a SharedElementTransition when matching [Hero] widgets
  /// are available.
  factory CardDetailScreenArgument.fromCard(WalletCard card) {
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

  /// Will load card details for the given attestationId. This implicitly disables the SharedElementTransition
  /// since the [WalletCard] is not immediately available to render on the [CardDetailScreen].
  factory CardDetailScreenArgument.fromId(String attestationId, LocalizedText cardTitle) {
    return CardDetailScreenArgument(
      cardId: attestationId,
      cardTitle: cardTitle,
    );
  }

  factory CardDetailScreenArgument.fromJson(Map<String, dynamic> json) => _$CardDetailScreenArgumentFromJson(json);
}
