import 'package:flutter/material.dart';

import '../../domain/model/bloc/network_error_state.dart';
import '../../navigation/secured_page_route.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'error_button_builder.dart';

export 'error_cta_style.dart';

class ErrorScreen extends StatelessWidget {
  final String title;
  final String description;
  final FitsWidthWidget primaryButton;
  final FitsWidthWidget? secondaryButton;
  final String? illustration;
  final List<Widget> actions;

  const ErrorScreen({
    required this.title,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    this.illustration,
    this.actions = const [CloseIconButton()],
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: WalletScrollbar(
          child: Column(
            children: [
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: title,
                      scrollController: PrimaryScrollController.maybeOf(context),
                      automaticallyImplyLeading: false,
                      actions: actions,
                    ),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: BodyText(description),
                      ),
                    ),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(vertical: 24),
                        child: PageIllustration(
                          asset: illustration ?? WalletAssets.svg_error_general,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              _buildBottomSection(context),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        children: [
          const Divider(height: 1),
          ConfirmButtons(
            forceVertical: !context.isLandscape,
            flipVertical: true,
            hideSecondaryButton: secondaryButton == null,
            secondaryButton: secondaryButton ?? const TertiaryButton(text: Text('' /* invisible placeholder */)),
            primaryButton: primaryButton,
          ),
        ],
      ),
    );
  }

  static void show(
    BuildContext context, {
    required ErrorScreenConfig config,
    bool secured = true,
  }) {
    final errorScreen = ErrorScreen(
      title: config.title,
      description: config.description,
      illustration: config.illustration,
      primaryButton: config.primaryButton,
      secondaryButton: config.secondaryButton,
      actions: config.actions,
    );
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(
              builder: (c) => errorScreen,
            )
          : MaterialPageRoute(
              builder: (c) => errorScreen,
            ),
    );
  }

  /// Shows the [ErrorScreen] with the most generic error message
  /// i.e. 'something went wrong' and a close button. Useful when
  /// we only want to communicate something went wrong without going
  /// into any specifics.
  static void showGeneric(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry, bool secured = true}) {
    show(
      context,
      secured: secured,
      config: ErrorScreenConfig(
        title: context.l10n.errorScreenGenericHeadline,
        description: style == ErrorCtaStyle.close
            ? context.l10n.errorScreenGenericDescriptionCloseVariant
            : context.l10n.errorScreenGenericDescription,
        illustration: WalletAssets.svg_error_general,
        primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
        secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
        actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
      ),
    );
  }

  /// Shows the [ErrorScreen] focussed on communicating
  /// a network related error. The error displayed to the user is
  /// based on the provided [NetworkErrorState], and defaults to
  /// 'something went wrong, check the internet and try again'
  /// when no [NetworkErrorState] is provided.
  static void showNetwork(
    BuildContext context, {
    bool secured = true,
    NetworkErrorState? networkError,
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    if (networkError?.hasInternet == false) {
      showNoInternet(context, style: style, secured: secured);
    } else {
      /// [networkError.statusCode] can eventually be used to show more specific errors
      show(
        context,
        secured: secured,
        config: ErrorScreenConfig(
          title: context.l10n.errorScreenServerHeadline,
          description: style == ErrorCtaStyle.close
              ? context.l10n.errorScreenServerDescriptionCloseVariant
              : context.l10n.errorScreenServerDescription,
          illustration: WalletAssets.svg_error_server_outage,
          primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
          secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
          actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
        ),
      );
    }
  }

  static void showNoInternet(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    bool secured = true,
  }) {
    show(
      context,
      secured: secured,
      config: ErrorScreenConfig(
        title: context.l10n.errorScreenNoInternetHeadline,
        description: style == ErrorCtaStyle.close
            ? context.l10n.errorScreenNoInternetDescriptionCloseVariant
            : context.l10n.errorScreenNoInternetDescription,
        illustration: WalletAssets.svg_error_no_internet,
        primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
        secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
        actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
      ),
    );
  }

  static void showDeviceIncompatible(BuildContext context) {
    show(
      context,
      secured: false,
      config: ErrorScreenConfig(
        title: context.l10n.errorScreenDeviceIncompatibleHeadline,
        description: context.l10n.errorScreenDeviceIncompatibleDescription,
        illustration: WalletAssets.svg_error_config_update,
        primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
          context,
          ErrorCtaStyle.close,
          onPressed: () {
            Navigator.pushNamedAndRemoveUntil(
              context,
              WalletRoutes.splashRoute,
              ModalRoute.withName(WalletRoutes.splashRoute),
            );
          },
        ),
        secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
        actions: [],
      ),
    );
  }

  static void showSessionExpired(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    bool secured = true,
  }) {
    show(
      context,
      secured: secured,
      config: ErrorScreenConfig(
        title: context.l10n.errorScreenSessionExpiredHeadline,
        description: style == ErrorCtaStyle.close
            ? context.l10n.errorScreenSessionExpiredDescriptionCloseVariant
            : context.l10n.errorScreenSessionExpiredDescription,
        illustration: WalletAssets.svg_error_session_expired,
        primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
        secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
        actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
      ),
    );
  }
}

/// Wrapper for all the info needed to render the [ErrorScreen].
class ErrorScreenConfig {
  final String title;
  final String description;
  final FitsWidthWidget primaryButton;
  final FitsWidthWidget? secondaryButton;
  final String illustration;
  final List<Widget> actions;

  ErrorScreenConfig({
    required this.title,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    required this.illustration,
    this.actions = const [CloseIconButton()],
  });
}
