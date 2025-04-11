import 'package:collection/collection.dart';
import 'package:flutter/material.dart';

import '../../../../../environment.dart';
import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/metadata/card_rendering.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../theme/base_wallet_theme.dart';
import '../../../../theme/wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/locale_extension.dart';
import '../../../../util/extension/object_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../decoration/shadow_decoration.dart';
import '../animated_fade_in.dart';
import '../animated_fade_out.dart';
import '../text/body_text.dart';
import '../text/headline_small_text.dart';
import '../utility/disable_text_scaling.dart';
import 'bottom_clip_shadow.dart';
import 'card_logo.dart';
import 'card_network_logo.dart';
import 'mock_card_background.dart';
import 'mock_card_holograph.dart';
import 'show_details_cta.dart';

// Fallback colors used when the issuer does not supply (both) a text and background color.
// The reason we fall back to these colors even if only one of them is missing is to guarantee
// sufficient contrast between the text- and background color.
const Color _kFallbackBgColor = Color(0xFFEEEFF7);
const Color _kFallbackTextColor = Color(0xFF152A62);

// Default card size constraints, configured so the card can expand vertically.
const _kCardSizeConstraints = BoxConstraints(maxWidth: 328, minHeight: 192);
const _kCardBorderRadius = WalletTheme.kBorderRadius12;
const _kCardContentPadding = 24.0;

class WalletCardItem extends StatefulWidget {
  /// The cards title
  final String title;

  /// The cards subtitle, rendered below the title
  final String? subtitle;

  /// The logo, rendered in the top right corner
  final Widget? logo;

  // The background, rendered behind the card's content
  final Widget? background;

  // The holograph, rendered on the right side between the background and content
  final Widget? holograph;

  // The textColor, used for the title, description and cta
  final Color? textColor;

  /// Callback that is triggered when the card is clicked
  ///
  /// 'View' CTA will be hidden if [onPressed] is null.
  final VoidCallback? onPressed;

  /// Show the title & subtitle, defaults to true.
  final bool showText;

  /// If the text should be scaled based on the device's textScaleFactor
  /// Worth disabling if widget is used as a thumbnail.
  final bool scaleText;

  /// Specify how to animate the 'show details' cta on the initial build
  final CtaAnimation? ctaAnimation;

  const WalletCardItem({
    required this.title,
    this.subtitle,
    this.logo,
    this.background,
    this.holograph,
    this.textColor,
    this.onPressed,
    this.showText = true,
    this.scaleText = true,
    this.ctaAnimation,
    super.key,
  });

  factory WalletCardItem.fromWalletCard(
    BuildContext context,
    WalletCard card, {
    VoidCallback? onPressed,
    CtaAnimation ctaAnimation = CtaAnimation.visible,
    bool scaleText = true,
    bool showText = true,
    Key? key,
  }) {
    return WalletCardItem(
      title: card.title.l10nValue(context),
      subtitle: card.summary.l10nValue(context),
      background: card.getL10nBackground(context),
      logo: card.getL10nLogo(context),
      textColor: card.getL10nTextColor(context),
      onPressed: onPressed,
      ctaAnimation: ctaAnimation,
      holograph: MockCardHolograph(docType: card.docType),
      scaleText: scaleText,
      showText: showText,
      key: key,
    );
  }

  @override
  State<WalletCardItem> createState() => _WalletCardItemState();

  static Widget buildShuttleCard(
    Animation<double> animation,
    WalletCard card, {
    CtaAnimation ctaAnimation = CtaAnimation.visible,
  }) {
    final scaleTween = TweenSequence<double>(
      <TweenSequenceItem<double>>[
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1, end: 1.05).chain(CurveTween(curve: Curves.easeIn)),
          weight: 30,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.05, end: 1.05),
          weight: 60,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.05, end: 1).chain(CurveTween(curve: Curves.easeInCubic)),
          weight: 10,
        ),
      ],
    );

    final perspectiveTween = TweenSequence<double>(
      <TweenSequenceItem<double>>[
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0, end: 0.2).chain(CurveTween(curve: Curves.easeInCubic)),
          weight: 20,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.2, end: 0.2),
          weight: 65,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.2, end: 0).chain(CurveTween(curve: Curves.decelerate)),
          weight: 15,
        ),
      ],
    );

    final VoidCallback? onPressed = switch (ctaAnimation) {
      CtaAnimation.fadeIn => () {},
      CtaAnimation.fadeOut => () {},
      CtaAnimation.visible => () {},
      CtaAnimation.invisible => null,
    };

    return AnimatedBuilder(
      animation: animation,
      child: Builder(
        builder: (context) {
          return WalletCardItem.fromWalletCard(
            context,
            card,
            ctaAnimation: ctaAnimation,
            onPressed: onPressed,
          );
        },
      ),
      builder: (context, child) {
        return Transform(
          alignment: FractionalOffset.center,
          transform: Matrix4.identity()
            ..scale(scaleTween.evaluate(animation))
            ..setEntry(3, 2, 0.001)
            ..rotateX(perspectiveTween.evaluate(animation)),
          child: child,
        );
      },
    );
  }
}

class _WalletCardItemState extends State<WalletCardItem> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(_onWidgetStateChanged));
  }

  @override
  void dispose() {
    _statesController.removeListener(_onWidgetStateChanged);
    _statesController.dispose();
    super.dispose();
  }

  void _onWidgetStateChanged() => setState(() {});

  @override
  Widget build(BuildContext context) {
    final themeWithUpdatedTextColor = context.theme.copyWith(
      textTheme: context.textTheme.apply(
        bodyColor: widget.textColor,
        displayColor: widget.textColor,
        decoration: _statesController.value.isPressedOrFocused ? TextDecoration.underline : null,
      ),
    );
    return Theme(
      data: themeWithUpdatedTextColor,
      child: DisableTextScaling(
        disableTextScaling: !widget.scaleText,
        child: FittedBox(
          child: ConstrainedBox(
            constraints: _kCardSizeConstraints,
            child: DecoratedBox(
              decoration: CardShadowDecoration(),
              child: Material(
                color: Colors.transparent,
                borderRadius: _kCardBorderRadius,
                clipBehavior: Clip.antiAlias,
                child: MergeSemantics(
                  child: Stack(
                    children: [
                      Positioned.fill(child: _buildBackground()),
                      _buildContent(context),
                      _buildPositionedShowDetailsCta(context),
                      Positioned.fill(child: _buildRippleAndFocus(context)),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBackground() {
    const fallBackBackground = DecoratedBox(decoration: BoxDecoration(color: _kFallbackBgColor));
    return Stack(
      fit: StackFit.expand,
      children: [
        widget.background ?? fallBackBackground,
        if (widget.holograph != null) Positioned(bottom: 24, right: 24, child: widget.holograph!),
        // Draw a subtle shadow overlay (3d effect) at the bottom.
        BottomClipShadow(radius: _kCardBorderRadius.bottomLeft.x),
      ],
    );
  }

  Widget _buildContent(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(_kCardContentPadding),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Semantics(attributedLabel: context.l10n.cardTitleSemanticsLabel.toAttributedString(context)),
                HeadlineSmallText(widget.title.takeIf((_) => widget.showText) ?? ''),
                const SizedBox(height: 4),
                BodyText(widget.subtitle.takeIf((_) => widget.showText) ?? ''),
                Opacity(
                  opacity: 0,
                  child: _buildShowDetailsCta(context),
                ), // guarantees correct spacing for the cta rendered at the bottom of the card
              ],
            ),
          ),
          SizedBox(width: widget.logo == null ? 0 : 16),
          widget.logo ?? const SizedBox.shrink(),
        ],
      ),
    );
  }

  Widget _buildShowDetailsCta(BuildContext context) {
    return Focus(
      canRequestFocus: false,
      descendantsAreFocusable: false, // Makes sure the cta doesn't receive separate focus
      child: Align(
        alignment: Alignment.centerLeft,
        child: IntrinsicWidth(
          child: ShowDetailsCta(
            text: Text(context.l10n.showDetailsCta),
            textColor: widget.textColor,
            onPressed: widget.onPressed,
            statesController: _statesController,
          ),
        ),
      ),
    );
  }

  Widget _buildPositionedShowDetailsCta(BuildContext context) {
    final showDetailsCta = widget.onPressed != null;
    if (!showDetailsCta) return const SizedBox.shrink();
    return Positioned(
      bottom: _kCardContentPadding,
      left: _kCardContentPadding,
      right: _kCardContentPadding,
      child: switch (widget.ctaAnimation) {
        null => widget.onPressed == null ? const SizedBox.shrink() : _buildShowDetailsCta(context),
        CtaAnimation.fadeIn => AnimatedFadeIn(child: _buildShowDetailsCta(context)),
        CtaAnimation.fadeOut => AnimatedFadeOut(child: _buildShowDetailsCta(context)),
        CtaAnimation.visible => _buildShowDetailsCta(context),
        CtaAnimation.invisible => const SizedBox.shrink(),
      },
    );
  }

  Widget _buildRippleAndFocus(BuildContext context) {
    return ExcludeSemantics(
      child: TextButton(
        style: context.theme.textButtonTheme.style?.copyWith(
          backgroundColor: const WidgetStatePropertyAll(Colors.transparent),
          padding: const WidgetStatePropertyAll(EdgeInsets.zero),
        ),
        statesController: _statesController,
        onPressed: widget.onPressed,
        child: const SizedBox.shrink(),
      ),
    );
  }
}

/// Helper methods to resolve the correct localisations for [WalletCard],
/// To maintain the mock we check for the [CardFront] and build the UI
/// based on that when it's available & a mock build.
extension WalletCardRenderExtension on WalletCard {
  CardRendering? getL10nRendering(BuildContext context) {
    // Find an exact locale match
    final matchingLocale = metadata.firstWhereOrNull(
      (metadata) {
        return metadata.language.matchesCurrentLocale(context) && metadata.rendering != null;
      },
    )?.rendering;
    if (matchingLocale != null) return matchingLocale;
    // Fall back on a language match
    final matchingLanguage = metadata.firstWhereOrNull(
      (metadata) {
        return metadata.language.matchesCurrentLanguage(context) && metadata.rendering != null;
      },
    )?.rendering;
    if (matchingLanguage != null) return matchingLanguage;
    // Fall back on the first specified rendering
    return metadata.firstOrNull?.rendering;
  }

  Color getL10nTextColor(BuildContext context) {
    final rendering = getL10nRendering(context);
    switch (rendering) {
      case null:
        return _kFallbackTextColor;
      case SimpleCardRendering():
        return rendering.textColor
                .takeIf((_) => rendering.bgColor != null /* guarantee contrast */ || Environment.isMockOrTest) ??
            _kFallbackTextColor;
    }
  }

  Widget getL10nBackground(BuildContext context) {
    if (Environment.mockRepositories) return MockCardBackground(docType: docType);
    final rendering = getL10nRendering(context);
    switch (rendering) {
      case null:
        return DecoratedBox(decoration: BoxDecoration(color: _kFallbackBgColor));
      case SimpleCardRendering():
        final bgColor = rendering.bgColor
                .takeIf((_) => rendering.textColor != null /* guarantee contrast */ || Environment.isMockOrTest) ??
            _kFallbackBgColor;
        return DecoratedBox(decoration: BoxDecoration(color: bgColor));
    }
  }

  Widget? getL10nLogo(BuildContext context) {
    final rendering = getL10nRendering(context);
    if (rendering == null) return null;

    switch (rendering) {
      case SimpleCardRendering():
        final logoUri = rendering.logoUri;
        if (logoUri == null) return null;
        if (Environment.isMockOrTest && logoUri.startsWith('assets/')) {
          return CardLogo(logo: logoUri, altText: rendering.logoAltText);
        }
        return CardNetworkLogo(uri: logoUri, altText: rendering.logoAltText);
    }
  }
}

enum CtaAnimation { fadeIn, fadeOut, visible, invisible }
