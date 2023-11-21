import 'package:wallet_core/core.dart';

import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';

/// Maps a [RequestedCard] to a [WalletCard] and enriches with (currently) hardcoded data.
class RequestedCardMapper extends Mapper<RequestedCard, WalletCard> {
  final Mapper<Card, WalletCard> _cardMapper;

  RequestedCardMapper(this._cardMapper);

  @override
  WalletCard map(RequestedCard input) => _cardMapper.map(
        Card(
          persistence: const CardPersistence.inMemory(),
          docType: input.docType,
          attributes: input.attributes,
        ),
      );
}
