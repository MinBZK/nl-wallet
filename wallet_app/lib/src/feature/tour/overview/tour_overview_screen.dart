import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/tour/tour_video.dart';
import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/bullet_list.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/wallet_scrollbar.dart';
import 'tour_video_data.dart';

class TourOverviewScreen extends StatelessWidget {
  const TourOverviewScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(child: _buildContent(context)),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: context.l10n.tourOverviewScreenTitle,
            scrollController: PrimaryScrollController.maybeOf(context),
          ),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: BodyText(context.l10n.tourOverviewScreenSubtitle),
            ),
          ),
          SliverSizedBox(height: 16),
          SliverDivider(),
          _buildVideoList(context),
        ],
      ),
    );
  }

  Widget _buildVideoList(BuildContext context) {
    final tourVideos = TourVideoData.videos(context);
    return SliverList.separated(
      itemBuilder: (context, index) => _buildVideoListItem(context, tourVideos[index]),
      itemCount: tourVideos.length,
      separatorBuilder: (context, index) => Divider(),
    );
  }

  Widget _buildVideoListItem(BuildContext context, TourVideo tourVideo) {
    return TextButton(
      onPressed: () => PlaceholderScreen.showGeneric(context),
      style: context.theme.iconButtonTheme.style?.copyWith(
        shape: WidgetStateProperty.all(
          const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.only(top: 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Stack(
              children: [
                _buildVideoThumb(context, tourVideo.videoThumb.l10nValue(context)),
                _buildPlayButton(context),
              ],
            ),
            SizedBox(height: 16),
            Text(
              tourVideo.title,
              style: context.textTheme.headlineMedium,
            ),
            BulletList(
              items: tourVideo.bulletPoints.split('\n'),
              icon: Icon(
                Icons.circle,
                color: context.colorScheme.onSurface,
                size: 6,
              ),
              rowCrossAxisAlignment: CrossAxisAlignment.start,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildVideoThumb(BuildContext context, String videoThumb) {
    return ClipRRect(
      borderRadius: WalletTheme.kBorderRadius12,
      child: AspectRatio(
        aspectRatio: 328 / 120,
        child: Image.asset(
          videoThumb,
          fit: BoxFit.cover,
          width: double.infinity,
        ),
      ),
    );
  }

  Widget _buildPlayButton(BuildContext context) {
    return Positioned.fill(
      child: Center(
        child: Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.all(Radius.circular(8)),
            color: context.colorScheme.surface,
          ),
          padding: EdgeInsets.all(8),
          child: Icon(
            Icons.play_arrow,
            color: context.colorScheme.primary,
            size: 24,
          ),
        ),
      ),
    );
  }
}
