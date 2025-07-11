import 'dart:io';

import 'package:flutter/material.dart';

import '../../../domain/usecase/update/observe_version_state_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/banner/notification_banner.dart';
import '../../common/widget/svg_or_image.dart';

const _kStoreLogoSize = 32.0;
const _kWarnDotSize = 21.0;

class UpdateBanner extends StatelessWidget {
  final VersionState versionState;

  const UpdateBanner({
    required this.versionState,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    switch (versionState) {
      case VersionStateOk():
      case VersionStateBlock():
        return const SizedBox.shrink();
      case VersionStateNotify():
        return NotificationBanner(
          leadingIcon: _buildStoreLogoWithWarnDot(context, false),
          title: context.l10n.updateBannerNotifyTitle,
          subtitle: context.l10n.updateBannerNotifyDescription,
          onTap: () => Navigator.pushNamed(context, WalletRoutes.updateInfoRoute),
        );
      case VersionStateRecommend():
        return NotificationBanner(
          leadingIcon: _buildStoreLogoWithWarnDot(context, true),
          title: context.l10n.updateBannerRecommendTitle,
          subtitle: context.l10n.updateBannerRecommendDescription,
          onTap: () => Navigator.pushNamed(context, WalletRoutes.updateInfoRoute),
        );
      case VersionStateWarn(:final timeUntilBlocked):
        return NotificationBanner(
          leadingIcon: _buildStoreLogoWithWarnDot(context, true),
          title: context.l10n.updateBannerWarnTitle,
          subtitle: _resolveWarnDescription(context, timeUntilBlocked),
          onTap: () => Navigator.pushNamed(context, WalletRoutes.updateInfoRoute),
        );
    }
  }

  Widget _buildStoreLogoWithWarnDot(BuildContext context, bool warn) {
    return Stack(
      clipBehavior: Clip.none,
      children: [
        _buildStoreLogo(),
        Positioned(
          width: _kWarnDotSize,
          height: _kWarnDotSize,
          right: _kWarnDotSize / -3,
          bottom: _kWarnDotSize / -3,
          child: AnimatedOpacity(
            opacity: warn ? 1 : 0,
            duration: kDefaultAnimationDuration,
            child: _buildWarnDot(context),
          ),
        ),
      ],
    );
  }

  Widget _buildStoreLogo() {
    return SizedBox(
      height: _kStoreLogoSize,
      width: _kStoreLogoSize,
      child: SvgOrImage(
        asset: Platform.isIOS ? WalletAssets.svg_app_store : WalletAssets.svg_play_store,
      ),
    );
  }

  Widget _buildWarnDot(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(shape: BoxShape.circle, color: context.colorScheme.surface),
      child: Icon(
        Icons.error_outlined,
        color: context.colorScheme.error,
        size: _kWarnDotSize - 1,
      ),
    );
  }

  String _resolveWarnDescription(BuildContext context, Duration timeUntilBlocked) {
    final days = timeUntilBlocked.inDays;
    final hours = timeUntilBlocked.inHours;
    if (days > 0) {
      return context.l10n.updateBannerWarnDescription(context.l10n.generalDays(days));
    } else if (hours > 0) {
      return context.l10n.updateBannerWarnDescription(context.l10n.generalHours(hours));
    } else {
      return context.l10n.updateBannerWarnDescription(context.l10n.generalHours(1));
    }
  }
}
