import 'package:flutter/material.dart';

import '../../domain/model/bloc/network_error_state.dart';
import '../../navigation/secured_page_route.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/page_illustration.dart';
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
          const Divider(),
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
    required ErrorScreen errorScreen,
    bool secured = true,
  }) {
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
  static void showGeneric(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    bool secured = true,
  }) {
    show(
      context,
      secured: secured,
      errorScreen: ErrorScreen.generic(context, style: style),
    );
  }

  factory ErrorScreen.generic(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    return ErrorScreen(
      title: context.l10n.errorScreenGenericHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenGenericDescriptionCloseVariant
          : context.l10n.errorScreenGenericDescription,
      illustration: WalletAssets.svg_error_general,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
      actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
    );
  }

  /// Shows the [ErrorScreen] focussed on communicating
  /// a network related error. The error displayed to the user is
  /// based on the provided [NetworkErrorState], and defaults to
  /// 'something went wrong, check the internet and try again'
  /// when no [NetworkErrorState] is provided.
  static void showNetwork(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    NetworkErrorState? networkError,
    bool secured = true,
  }) {
    show(
      context,
      secured: secured,
      errorScreen: ErrorScreen.network(
        context,
        style: style,
        networkError: networkError,
      ),
    );
  }

  factory ErrorScreen.network(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    NetworkErrorState? networkError,
  }) {
    if (networkError?.hasInternet == false) {
      return ErrorScreen.noInternet(context, style: style);
    } else {
      return ErrorScreen.serverOutage(context, style: style);
    }
  }

  factory ErrorScreen.serverOutage(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    return ErrorScreen(
      title: context.l10n.errorScreenServerHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenServerDescriptionCloseVariant
          : context.l10n.errorScreenServerDescription,
      illustration: WalletAssets.svg_error_server_outage,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
      actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
    );
  }

  factory ErrorScreen.noInternet(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    return ErrorScreen(
      title: context.l10n.errorScreenNoInternetHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenNoInternetDescriptionCloseVariant
          : context.l10n.errorScreenNoInternetDescription,
      illustration: WalletAssets.svg_error_no_internet,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
      actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
    );
  }

  static void showDeviceIncompatible(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    bool secured = true,
  }) {
    show(
      context,
      secured: secured,
      errorScreen: ErrorScreen.deviceIncompatible(context),
    );
  }

  factory ErrorScreen.deviceIncompatible(
    BuildContext context,
  ) {
    return ErrorScreen(
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
      errorScreen: ErrorScreen.sessionExpired(context, style: style),
    );
  }

  factory ErrorScreen.sessionExpired(
    BuildContext context, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    return ErrorScreen(
      title: context.l10n.errorScreenSessionExpiredHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenSessionExpiredDescriptionCloseVariant
          : context.l10n.errorScreenSessionExpiredDescription,
      illustration: WalletAssets.svg_error_session_expired,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
      actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
    );
  }
}
