import 'package:freezed_annotation/freezed_annotation.dart';

import '../../domain/model/card/wallet_card.dart';
import '../../domain/model/update/version_state.dart';

part 'wallet_banner.freezed.dart';

/// Represents the different types of banners that can be displayed within the wallet.
@freezed
sealed class WalletBanner with _$WalletBanner {
  /// A banner indicating that a new version of the application is available.
  ///
  /// Requires a [VersionState] to provide details about the update.
  const factory WalletBanner.updateAvailable({
    required VersionState state,
  }) = UpdateAvailableBanner;

  /// A banner suggesting that the user take a tour of the app's features.
  const factory WalletBanner.tourSuggestion() = TourSuggestionBanner;

  /// A banner warning the user that a card will expire soon.
  const factory WalletBanner.cardExpiresSoon({
    required WalletCard card,
    required DateTime expiresAt,
  }) = CardExpiresSoonBanner;

  /// A banner warning the user that a card has expired.
  const factory WalletBanner.cardExpired({
    required WalletCard card,
  }) = CardExpiredBanner;
}
