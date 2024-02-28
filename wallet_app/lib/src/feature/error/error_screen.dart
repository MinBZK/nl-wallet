import 'package:flutter/material.dart';

import '../../domain/model/bloc/network_error_state.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/sheet/help_sheet.dart';
import '../common/widget/wallet_app_bar.dart';
import 'error_page.dart';

class ErrorScreen extends StatelessWidget {
  final String? illustration;
  final String title;
  final String headline;
  final String description;
  final String primaryActionText;
  final String? secondaryActionText;
  final VoidCallback onPrimaryActionPressed;
  final VoidCallback? onSecondaryActionPressed;

  const ErrorScreen({
    required this.title,
    required this.headline,
    required this.description,
    required this.primaryActionText,
    required this.onPrimaryActionPressed,
    this.illustration,
    this.secondaryActionText,
    this.onSecondaryActionPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: Text(title),
        automaticallyImplyLeading: false,
        actions: const [CloseButton()],
      ),
      body: ErrorPage(
        illustration: illustration,
        headline: headline,
        description: description,
        primaryActionText: primaryActionText,
        secondaryActionText: secondaryActionText,
        onPrimaryActionPressed: onPrimaryActionPressed,
        onSecondaryActionPressed: onSecondaryActionPressed,
      ),
    );
  }

  static void show(
    BuildContext context, {
    bool secured = true,
    required String title,
    required String headline,
    required String description,
    required String primaryActionText,
    required VoidCallback onPrimaryActionPressed,
    String? illustration,
    String? secondaryActionText,
    VoidCallback? onSecondaryActionPressed,
  }) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(
              builder: (c) => ErrorScreen(
                title: title,
                headline: headline,
                description: description,
                illustration: illustration,
                primaryActionText: primaryActionText,
                onPrimaryActionPressed: onPrimaryActionPressed,
                secondaryActionText: secondaryActionText,
                onSecondaryActionPressed: onSecondaryActionPressed,
              ),
            )
          : MaterialPageRoute(
              builder: (c) => ErrorScreen(
                title: title,
                headline: headline,
                description: description,
                illustration: illustration,
                primaryActionText: primaryActionText,
                onPrimaryActionPressed: onPrimaryActionPressed,
                secondaryActionText: secondaryActionText,
                onSecondaryActionPressed: onSecondaryActionPressed,
              ),
            ),
    );
  }

  /// Shows the [ErrorScreen] with the most generic error message
  /// i.e. 'something went wrong' and a close button. Useful when
  /// we only want to communicate something went wrong without going
  /// into any specifics.
  static void showGeneric(BuildContext context, {String? title, bool secured = true}) {
    show(
      context,
      secured: secured,
      title: title ?? context.l10n.errorScreenGenericTitle,
      headline: context.l10n.errorScreenGenericHeadline,
      description: context.l10n.errorScreenGenericDescription,
      illustration: WalletAssets.illustration_general_error,
      primaryActionText: context.l10n.errorScreenGenericCloseCta,
      secondaryActionText: context.l10n.errorScreenGeneralHelpCta,
      onPrimaryActionPressed: () => Navigator.pop(context),
      onSecondaryActionPressed: () => HelpSheet.show(context),
    );
  }

  /// Shows the [ErrorScreen] focussed on communicating
  /// a network related error. The error displayed to the user is
  /// based on the provided [NetworkErrorState], and defaults to
  /// 'something went wrong, check the internet and try again'
  /// when no [NetworkErrorState] is provided.
  static void showNetwork(BuildContext context, {String? title, bool secured = true, NetworkErrorState? networkError}) {
    if (networkError?.hasInternet == false) {
      showNoInternet(context, title: title, secured: secured);
    } else {
      //TODO: We eventually want to select different copy based on the provided [NetworkError]s statusCode.
      show(
        context,
        secured: secured,
        title: title ?? context.l10n.errorScreenServerTitle,
        headline: context.l10n.errorScreenServerHeadline,
        description: context.l10n.errorScreenServerDescription,
        illustration: WalletAssets.illustration_server_error,
        primaryActionText: context.l10n.errorScreenServerCloseCta,
        secondaryActionText: context.l10n.errorScreenServerHelpCta,
        onPrimaryActionPressed: () => Navigator.pop(context),
        onSecondaryActionPressed: () => HelpSheet.show(context),
      );
    }
  }

  static void showNoInternet(BuildContext context, {String? title, bool secured = true}) {
    show(
      context,
      secured: secured,
      title: title ?? context.l10n.errorScreenNoInternetTitle,
      headline: context.l10n.errorScreenNoInternetHeadline,
      description: context.l10n.errorScreenNoInternetDescription,
      illustration: WalletAssets.illustration_no_internet_error,
      primaryActionText: context.l10n.generalRetry,
      onPrimaryActionPressed: () => Navigator.pop(context),
    );
  }
}
