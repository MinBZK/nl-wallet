import 'package:flutter/material.dart';

import '../../domain/model/bloc/network_error_state.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/page/page_illustration.dart';
import '../common/sheet/error_details_sheet.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/text/body_text.dart';
import 'error_button_builder.dart';
import 'error_cta_style.dart';

export 'error_cta_style.dart';

class ErrorScreen extends StatelessWidget {
  final String? illustration;
  final String headline;
  final String description;
  final FitsWidthWidget primaryButton;
  final FitsWidthWidget? secondaryButton;
  final List<Widget> actions;

  const ErrorScreen({
    required this.headline,
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
        child: Scrollbar(
          child: CustomScrollView(
            slivers: [
              SliverWalletAppBar(
                title: headline,
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
              SliverFillRemaining(
                hasScrollBody: false,
                fillOverscroll: true,
                child: _buildBottomSection(),
              )
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection() {
    Widget content;
    if (secondaryButton == null) {
      content = primaryButton;
    } else {
      content = ConfirmButtons(
        forceVertical: true,
        flipVertical: true,
        secondaryButton: secondaryButton!,
        primaryButton: primaryButton,
      );
    }
    return Align(
      alignment: Alignment.bottomCenter,
      child: content,
    );
  }

  static void show(
    BuildContext context, {
    bool secured = true,
    required String headline,
    required String description,
    String? illustration,
    required FitsWidthWidget primaryButton,
    FitsWidthWidget? secondaryButton,
    List<Widget> actions = const [CloseIconButton()],
  }) {
    final errorScreen = ErrorScreen(
      headline: headline,
      description: description,
      illustration: illustration,
      primaryButton: primaryButton,
      secondaryButton: secondaryButton,
      actions: actions,
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
  static void showGeneric(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.close, bool secured = true}) {
    show(
      context,
      secured: secured,
      headline: context.l10n.errorScreenGenericHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenGenericDescriptionCloseVariant
          : context.l10n.errorScreenGenericDescription,
      illustration: WalletAssets.svg_error_general,
      primaryButton: PrimaryButton(
        text: Text(style == ErrorCtaStyle.close ? context.l10n.errorScreenGenericCloseCta : context.l10n.generalRetry),
        icon: Icon(style == ErrorCtaStyle.close ? Icons.close_outlined : Icons.replay_outlined),
        onPressed: () => Navigator.pop(context),
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.generalShowDetailsCta),
        onPressed: () => ErrorDetailsSheet.show(context),
      ),
    );
  }

  /// Shows the [ErrorScreen] focussed on communicating
  /// a network related error. The error displayed to the user is
  /// based on the provided [NetworkErrorState], and defaults to
  /// 'something went wrong, check the internet and try again'
  /// when no [NetworkErrorState] is provided.
  static void showNetwork(BuildContext context, {bool secured = true, NetworkErrorState? networkError}) {
    if (networkError?.hasInternet == false) {
      showNoInternet(context, secured: secured);
    } else {
      /// [networkError.statusCode] can eventually be used to show more specific errors
      show(
        context,
        secured: secured,
        headline: context.l10n.errorScreenServerHeadline,
        description: context.l10n.errorScreenServerDescription,
        illustration: WalletAssets.svg_error_server_outage,
        primaryButton: PrimaryButton(
          text: Text(context.l10n.errorScreenServerCloseCta),
          onPressed: () => Navigator.pop(context),
        ),
        secondaryButton: TertiaryButton(
          text: Text(context.l10n.errorScreenServerHelpCta),
          onPressed: () => ErrorDetailsSheet.show(context),
        ),
      );
    }
  }

  static void showNoInternet(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry, bool secured = true}) {
    show(
      context,
      secured: secured,
      headline: context.l10n.errorScreenNoInternetHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenNoInternetDescriptionCloseVariant
          : context.l10n.errorScreenNoInternetDescription,
      illustration: WalletAssets.svg_error_no_internet,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(context, style),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.generalShowDetailsCta),
        icon: const Icon(Icons.info_outline_rounded),
        onPressed: () => ErrorDetailsSheet.show(context),
      ),
      actions: style == ErrorCtaStyle.close ? const [CloseIconButton()] : [],
    );
  }
}
