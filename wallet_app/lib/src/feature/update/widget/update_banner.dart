import 'dart:io';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../../domain/usecase/update/observe_version_state_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../theme/base_wallet_theme.dart';
import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/svg_or_image.dart';
import '../../common/widget/text/body_text.dart';

const _kStoreLogoSize = 32.0;
const _kWarnDotSize = 21.0;
const _kSlideInBounceAnimationDuration = Duration(seconds: 1);

class UpdateBanner extends StatefulWidget {
  final EdgeInsets padding;

  const UpdateBanner({this.padding = EdgeInsets.zero, super.key});

  @override
  State<UpdateBanner> createState() => _UpdateBannerState();
}

class _UpdateBannerState extends State<UpdateBanner> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _slideInBounceAnimation;
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();

    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(_updateState));

    _animationController = AnimationController(
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
    ]).animate(_animationController);
  }

  void _updateState() => setState(() {});

  @override
  void dispose() {
    _statesController.removeListener(_updateState);
    _statesController.dispose();
    _animationController.dispose();
    super.dispose();
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
              _animationController.forward();
              return _buildBanner(
                context,
                title: context.l10n.updateBannerNotifyTitle,
                subtitle: context.l10n.updateBannerNotifyDescription,
              );
            case VersionStateRecommend():
              _animationController.forward();
              return _buildBanner(
                context,
                title: context.l10n.updateBannerRecommendTitle,
                subtitle: context.l10n.updateBannerRecommendDescription,
                warn: true,
              );
            case VersionStateWarn():
              _animationController.forward();
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
    return Padding(
      padding: widget.padding,
      child: AnimatedBuilder(
        animation: _animationController,
        builder: (context, child) {
          return Transform.translate(
            offset: Offset(0, _slideInBounceAnimation.value),
            child: child,
          );
        },
        child: Material(
          color: context.colorScheme.tertiaryContainer,
          shape: RoundedRectangleBorder(
            borderRadius: WalletTheme.kBorderRadius12,
            side: context.theme.elevatedButtonTheme.style?.side?.resolve(_statesController.value) ?? BorderSide.none,
          ),
          child: InkWell(
            statesController: _statesController,
            borderRadius: WalletTheme.kBorderRadius12,
            onTap: () => Navigator.pushNamed(context, WalletRoutes.updateInfoRoute),
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
      ),
    );
  }

  Widget _buildForwardIcon() {
    return Icon(
      Icons.arrow_forward_outlined,
      size: context.theme.iconTheme.size! * (_statesController.value.isFocused ? 1.25 : 1.0),
    );
  }

  Widget _buildTextContent(BuildContext context, String title, String subtitle) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        BodyText(
          title,
          style: context.textTheme.labelLarge?.copyWith(
            decoration: _statesController.value.isFocused ? TextDecoration.underline : null,
          ),
        ),
        BodyText(
          subtitle,
          style: context.textTheme.bodyLarge?.copyWith(
            decoration: _statesController.value.isFocused ? TextDecoration.underline : null,
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
