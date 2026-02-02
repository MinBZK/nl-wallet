import 'dart:async';

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:rxdart/rxdart.dart';

import '../../../domain/usecase/notification/observe_dashboard_notifications_usecase.dart';
import '../../../domain/usecase/tour/observe_show_tour_banner_usecase.dart';
import '../../../domain/usecase/update/observe_version_state_usecase.dart';
import '../../../util/extension/list_extension.dart';
import '../../../util/extension/object_extension.dart';
import '../../../wallet_constants.dart';
import '../wallet_banner.dart';

const _kTourBanner = TourSuggestionBanner();

class BannerCubit extends Cubit<List<WalletBanner>> {
  final ObserveVersionStateUsecase _observeVersionStateUsecase;
  final ObserveShowTourBannerUseCase _observeShowTourBannerUseCase;
  final ObserveDashboardNotificationsUseCase _observeDashboardNotificationsUseCase;

  StreamSubscription? _bannerStreamSubscription;

  BannerCubit(
    this._observeShowTourBannerUseCase,
    this._observeVersionStateUsecase,
    this._observeDashboardNotificationsUseCase,
  ) : super([]) {
    _observeBannerState();
  }

  FutureOr<void> _observeBannerState() async {
    final bannersStream = CombineLatestStream.combine3(
      _observeVersionStateUsecase.invoke(),
      _observeShowTourBannerUseCase.invoke(),
      _observeDashboardNotificationsUseCase.invoke(),
      (versionState, showTour, notifications) async* {
        final banners = [
          _resolveUpdateBanner(versionState),
          ...notifications,
          _kTourBanner.takeIf((_) => showTour && state.contains(_kTourBanner)),
        ].nonNullsList;
        yield banners;

        // If TourBanner is not already visible, show it after 3 seconds (PVW-1750)
        if (showTour && !banners.contains(_kTourBanner)) {
          await Future.delayed(const Duration(seconds: 3));
          yield [...banners, _kTourBanner];
        }
      },
    ).flatMap((banners) => banners).debounceTime(kDefaultAnimationDuration);

    _bannerStreamSubscription = bannersStream.listen(emit);
  }

  WalletBanner? _resolveUpdateBanner(VersionState versionState) {
    if (versionState is VersionStateOk) return null;
    if (versionState is VersionStateBlock) return null;
    return UpdateAvailableBanner(state: versionState);
  }

  @override
  Future<void> close() async {
    await _bannerStreamSubscription?.cancel();
    return super.close();
  }
}
