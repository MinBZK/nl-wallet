import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/tour/tour_video.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/bullet_list.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../../error/error_page.dart';
import '../video/argument/tour_video_screen_argument.dart';
import 'bloc/tour_overview_bloc.dart';

class TourOverviewScreen extends StatelessWidget {
  const TourOverviewScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: BlocBuilder<TourOverviewBloc, TourOverviewState>(
                builder: (context, state) {
                  switch (state) {
                    case TourInitial():
                    case TourLoading():
                      return const CenteredLoadingIndicator();
                    case TourLoaded():
                      return _buildContent(context, state);
                    case TourLoadFailed():
                      return ErrorPage.generic(
                        context,
                        onPrimaryActionPressed: () => context.read<TourOverviewBloc>().add(const FetchVideosEvent()),
                        style: ErrorCtaStyle.retry,
                      );
                  }
                },
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context, TourLoaded state) {
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
          const SliverSizedBox(height: 16),
          const SliverDivider(),
          _buildVideoList(context, state),
        ],
      ),
    );
  }

  Widget _buildVideoList(BuildContext context, TourLoaded state) {
    return SliverList.separated(
      itemBuilder: (context, index) => _buildVideoListItem(context, state.tourVideos[index]),
      itemCount: state.tourVideos.length,
      separatorBuilder: (context, index) => const Divider(),
    );
  }

  Widget _buildVideoListItem(BuildContext context, TourVideo tourVideo) {
    return TextButton(
      onPressed: () => _handleVideoButtonPressed(context, tourVideo),
      style: context.theme.iconButtonTheme.style?.copyWith(
        shape: WidgetStateProperty.all(
          const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
        ),
      ),
      isSemanticButton: null,
      child: Padding(
        padding: const EdgeInsets.only(top: 16),
        child: Semantics(
          attributedLabel: context.l10n
              .tourOverviewScreenItemWCAGLabel(tourVideo.title.l10nValue(context))
              .toAttributedString(context),
          explicitChildNodes: true,
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
              const SizedBox(height: 16),
              TitleText(
                tourVideo.title.l10nValue(context),
              ),
              const SizedBox(height: 16),
              BulletList(
                items: tourVideo.bulletPoints.l10nValue(context).split('\n'),
                icon: Icon(
                  Icons.circle,
                  color: context.colorScheme.onSurface,
                  size: 6,
                ),
                rowCrossAxisAlignment: CrossAxisAlignment.start,
              ),
              const SizedBox(height: 16),
            ],
          ),
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
            borderRadius: const BorderRadius.all(Radius.circular(8)),
            color: context.colorScheme.surface,
          ),
          padding: const EdgeInsets.all(8),
          child: Icon(
            Icons.play_arrow,
            color: context.colorScheme.primary,
            size: 24,
          ),
        ),
      ),
    );
  }

  void _handleVideoButtonPressed(BuildContext context, TourVideo tourVideo) {
    final argument = TourVideoScreenArgument(
      subtitleUrl: tourVideo.subtitleUrl.l10nValue(context),
      videoUrl: tourVideo.videoUrl.l10nValue(context),
      videoTitle: tourVideo.title.l10nValue(context),
    );
    Navigator.restorablePushNamed(context, WalletRoutes.tourVideoRoute, arguments: argument.toMap());
  }
}
