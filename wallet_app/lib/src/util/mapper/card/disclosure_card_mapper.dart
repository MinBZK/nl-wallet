import 'package:wallet_core/core.dart';

import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';

/// Maps a [DisclosureCard] to a [WalletCard] and enriches with (currently) hardcoded data.
class DisclosureCardMapper extends Mapper<DisclosureCard, WalletCard> {
  final Mapper<Card, WalletCard> _cardMapper;

  DisclosureCardMapper(this._cardMapper);

  @override
  WalletCard map(DisclosureCard input) => _cardMapper.map(
        Card(
          persistence: const CardPersistence.inMemory(),
          docType: input.docType,
          attributes: input.attributes,
          issuer: input.issuer,
        ),
      );
}
