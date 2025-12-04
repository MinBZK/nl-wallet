import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../util/extension/card_display_metadata_list_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/mapper/card/summary_mapper.dart';
import '../attribute/attribute.dart';
import '../organization.dart';
import 'metadata/card_display_metadata.dart';
import 'status/card_status.dart';

part 'wallet_card.freezed.dart';
part 'wallet_card.g.dart';

@freezed
abstract class WalletCard with _$WalletCard {
  const factory WalletCard({
    /// ID of the attestation, null when the card is not persisted in the database
    String? attestationId,

    /// Type of document
    required String attestationType,

    /// Organization that issued this card
    required Organization issuer,

    /// Card status (e.g. valid, expired, revoked)
    required CardStatus status,

    /// Time from which the card is valid
    required DateTime validFrom,

    /// Time until the card is valid (expiry date)
    required DateTime validUntil,

    /// Data attributes stored in the card
    required List<DataAttribute> attributes,

    /// Card display metadata for UI rendering
    @Default([]) List<CardDisplayMetadata> metadata,
  }) = _WalletCard;

  const WalletCard._();

  factory WalletCard.fromJson(Map<String, dynamic> json) => _$WalletCardFromJson(json);

  /// Indicates whether the card is persisted in the database.
  bool get isPersisted => attestationId != null;

  LocalizedText get title => metadata.name ?? ''.untranslated;

  LocalizedText get description => metadata.description ?? ''.untranslated;

  LocalizedText get summary => CardSummaryMapper().map(this);
}
