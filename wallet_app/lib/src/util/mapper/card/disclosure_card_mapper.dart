import 'package:wallet_core/core.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../mapper.dart';

/// Maps a [DisclosureCard] to a [WalletCard] and enriches with (currently) hardcoded data.
class DisclosureCardMapper extends Mapper<DisclosureCard, WalletCard> {
  final Mapper<Attestation, WalletCard> _cardMapper;

  DisclosureCardMapper(this._cardMapper);

  @override
  WalletCard map(DisclosureCard input) => _cardMapper.map(
        Attestation(
          identity: const AttestationIdentity.ephemeral(),
          attestationType: input.docType,
          displayMetadata: input.displayMetadata,
          attributes: input.attributes,
          issuer: input.issuer,
        ),
      );
}
