import 'dart:io';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../domain/usecase/update/observe_version_state_usecase.dart';
import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/svg_or_image.dart';
import '../../common/widget/text/body_text.dart';

const _kStoreLogoSize = 32.0;
const _kWarnDotSize = 21.0;
const _kForwardIconSize = 16.0;
const _kSlideInBounceAnimationDuration = Duration(seconds: 1);

class UpdateBanner extends StatefulWidget {
  const UpdateBanner({super.key});

  @override
  State<UpdateBanner> createState() => _UpdateBannerState();
}

class _UpdateBannerState extends State<UpdateBanner> with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _slideInBounceAnimation;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      duration: _kSlideInBounceAnimationDuration,
      vsync: this,
    );

    _slideInBounceAnimation = TweenSequence<double>([
      TweenSequenceItem(
        tween: Tween<double>(begin: -300, end: 10).chain(CurveTween(curve: Curves.easeIn)), // Slide down
        weight: 60, // Major slide
      ),
      TweenSequenceItem(
        tween: Tween<double>(begin: 10, end: -10).chain(CurveTween(curve: Curves.easeInOut)), // Bounce up
        weight: 20,
      ),
      TweenSequenceItem(
        tween: Tween<double>(begin: -10, end: 0).chain(CurveTween(curve: Curves.easeInOut)), // Settle down
        weight: 20,
      ),
    ]).animate(_controller);
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedSize(
      duration: kDefaultAnimationDuration,
      curve: Curves.easeInOut,
      child: StreamBuilder<VersionState>(
        stream: context.read<ObserveVersionStateUsecase>().invoke(),
        builder: (context, state) {
          if (!state.hasData) return const SizedBox.shrink();
          final versionState = state.data!;
          switch (versionState) {
            case VersionStateOk():
              return const SizedBox.shrink();
            case VersionStateNotify():
              _controller.forward(from: 0);
              return _buildBanner(
                context,
                title: context.l10n.updateBannerNotifyTitle,
                subtitle: context.l10n.updateBannerNotifyDescription,
              );
            case VersionStateRecommend():
              _controller.forward(from: 0);
              return _buildBanner(
                context,
                title: context.l10n.updateBannerRecommendTitle,
                subtitle: context.l10n.updateBannerRecommendDescription,
                warn: true,
              );
            case VersionStateWarn():
              _controller.forward(from: 0);
              return _buildBanner(
                context,
                title: context.l10n.updateBannerWarnTitle,
                subtitle: _resolveWarnDescription(context, versionState.timeUntilBlocked),
                warn: true,
              );
            case VersionStateBlock():
              return const SizedBox.shrink();
          }
        },
      ),
    );
  }

  Widget _buildBanner(BuildContext context, {required String title, required String subtitle, bool warn = false}) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Transform.translate(
          offset: Offset(0, _slideInBounceAnimation.value),
          child: child,
        );
      },
      child: Material(
        color: context.colorScheme.tertiaryContainer,
        borderRadius: WalletTheme.kBorderRadius12,
        child: InkWell(
          borderRadius: WalletTheme.kBorderRadius12,
          onTap: () {},
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                Stack(
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
                ),
                const SizedBox(width: 12),
                Expanded(child: _buildTextContent(context, title, subtitle)),
                const SizedBox(width: 12),
                _buildForwardIcon(),
              ],
            ),
          ),
        ),
      ),
    );
  }

  SizedBox _buildForwardIcon() {
    return const SizedBox(
      width: _kForwardIconSize,
      height: _kForwardIconSize,
      child: Icon(Icons.arrow_forward_outlined),
    );
  }

  Widget _buildTextContent(BuildContext context, String title, String subtitle) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        BodyText(title, style: context.textTheme.labelLarge),
        BodyText(subtitle),
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
      decoration: const BoxDecoration(shape: BoxShape.circle, color: Colors.white),
      child: Icon(
        Icons.error_outlined,
        color: context.colorScheme.error,
        size: _kWarnDotSize - 1 /* -1 for white outline */,
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
