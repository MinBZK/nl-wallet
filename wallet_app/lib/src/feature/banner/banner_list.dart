import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../wallet_constants.dart';
import '../tour/widget/tour_banner.dart';
import '../update/widget/update_banner.dart';
import 'cubit/banner_cubit.dart';
import 'wallet_banner.dart';

/// A widget that displays a list of [WalletBanner]s with animations.
///
/// This widget listens to a [BannerCubit] for updates to the list of banners
class BannerList extends StatefulWidget {
  const BannerList({super.key});

  @override
  State<BannerList> createState() => _BannerListState();
}

class _BannerListState extends State<BannerList> {
  /// A global key used to control the [AnimatedListState].
  final GlobalKey<AnimatedListState> _listKey = GlobalKey<AnimatedListState>();

  /// The current list of banners being displayed.
  final List<WalletBanner> _banners = [];

  @override
  void initState() {
    super.initState();
    // Cubits only emit new states, so make sure initial state is in sync as [BannerCubit] outlives this widget.
    final banners = context.read<BannerCubit>().state;
    if (banners.isNotEmpty) _updateAnimatedList(banners);
  }

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<BannerCubit, List<WalletBanner>>(
      listener: (context, newBanners) => _updateAnimatedList(newBanners),
      builder: (context, banners) {
        return AnimatedList.separated(
          key: _listKey,
          padding: EdgeInsets.only(left: 16, top: banners.isEmpty ? 0 : 16, right: 16),
          shrinkWrap: true,
          initialItemCount: _banners.length,
          itemBuilder: (context, index, animation) {
            return _buildBannerItem(_banners[index], animation);
          },
          separatorBuilder: (BuildContext context, int index, Animation<double> animation) {
            return SizeTransition(sizeFactor: animation, child: const SizedBox(height: 8));
          },
          removedSeparatorBuilder: (BuildContext context, int index, Animation<double> animation) {
            return SizeTransition(sizeFactor: animation, child: const SizedBox(height: 8));
          },
        );
      },
    );
  }

  void _updateAnimatedList(List<WalletBanner> newBanners) {
    // Remove banners not in newBanners
    for (int i = _banners.length - 1; i >= 0; i--) {
      if (!newBanners.contains(_banners[i])) {
        final removedBanner = _banners.removeAt(i);
        _listKey.currentState?.removeItem(
          i,
          (context, animation) => _buildBannerItem(removedBanner, animation),
          duration: kDefaultAnimationDuration,
        );
      }
    }

    // Insert banners that are new
    for (int i = 0; i < newBanners.length; i++) {
      if (i >= _banners.length || _banners[i] != newBanners[i]) {
        _banners.insert(i, newBanners[i]);
        _listKey.currentState?.insertItem(
          i,
          duration: kDefaultAnimationDuration,
        );
      }
    }

    // If order changed or lists are somehow still not equal, reset the list.
    if (!const ListEquality().equals(_banners, newBanners)) {
      // Remove all
      _listKey.currentState?.removeAllItems(
        (context, animation) => const SizedBox.shrink(),
        duration: Duration.zero,
      );
      _banners.clear();

      // Add all [newBanners]
      for (int i = 0; i < newBanners.length; i++) {
        _banners.insert(i, newBanners[i]);
        _listKey.currentState?.insertItem(
          i,
          duration: kDefaultAnimationDuration,
        );
      }
    }
  }

  Widget _buildBannerItem(WalletBanner banner, Animation<double> animation) {
    Widget bannerWidget;
    switch (banner) {
      case UpdateAvailableBanner():
        bannerWidget = UpdateBanner(versionState: banner.state);
      case TourSuggestionBanner():
        bannerWidget = const TourBanner();
    }

    return FadeTransition(
      opacity: animation,
      child: SizeTransition(
        sizeFactor: animation.drive(CurveTween(curve: Curves.easeInOutCubic)),
        child: bannerWidget,
      ),
    );
  }
}
