import 'package:equatable/equatable.dart';

import '../../domain/model/update/version_state.dart';

/// Represents the different types of banners that can be displayed within the wallet.
sealed class WalletBanner extends Equatable {
  const WalletBanner();
}

/// A banner indicating that a new version of the application is available.
class UpdateAvailableBanner extends WalletBanner {
  final VersionState state;

  /// Creates an [UpdateAvailableBanner].
  ///
  /// Requires a [VersionState] to provide details about the update.
  const UpdateAvailableBanner({required this.state});

  @override
  List<Object?> get props => [state];
}

/// A banner suggesting that the user take a tour of the app's features.
class TourSuggestionBanner extends WalletBanner {
  const TourSuggestionBanner();

  @override
  List<Object?> get props => [];
}
