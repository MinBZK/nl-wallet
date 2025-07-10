import 'package:flutter/material.dart';

import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/banner/notification_banner.dart';
import '../../common/widget/svg_or_image.dart';

const _kTourIconSize = 32.0;

class TourBanner extends StatelessWidget {
  const TourBanner({super.key});

  @override
  Widget build(BuildContext context) {
    return NotificationBanner(
      leadingIcon: _buildTourIcon(),
      title: context.l10n.tourBannerTitle,
      onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.tourOverviewRoute),
    );
  }

  Widget _buildTourIcon() {
    return const SizedBox(
      height: _kTourIconSize,
      width: _kTourIconSize,
      child: SvgOrImage(
        asset: WalletAssets.svg_tour_icon,
      ),
    );
  }
}
