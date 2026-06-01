import 'package:flutter/material.dart';

import '../../domain/model/result/application_error.dart';
import '../../navigation/secured_page_route.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import 'error_page.dart';

export 'error_cta_style.dart';

const _kDefaultActions = [CloseIconButton()];

class ErrorScreen extends StatelessWidget {
  /// The title shown at the top of the page.
  final String title;

  /// The description text providing more details about the error.
  final String description;

  /// The main action button shown at the bottom of the page.
  final FitsWidthWidget primaryButton;

  /// An optional secondary action button shown below the [primaryButton].
  final FitsWidthWidget? secondaryButton;

  /// An optional SVG illustration asset path to display in the center of the page.
  final String? illustration;

  /// A list of actions shown at the top of the screen.
  final List<Widget> actions;

  const ErrorScreen({
    required this.title,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    this.illustration,
    this.actions = _kDefaultActions,
    super.key,
  });

  /// Creates an [ErrorScreen] from an existing [ErrorPage].
  ///
  /// Optionally, [actions] can be provided to customize the app bar actions.
  /// Defaults to a close button.
  factory ErrorScreen.fromPage(ErrorPage page, {List<Widget>? actions}) {
    return ErrorScreen(
      title: page.title,
      description: page.description,
      primaryButton: page.primaryButton,
      secondaryButton: page.secondaryButton,
      illustration: page.illustration,
      actions: actions ?? _kDefaultActions,
    );
  }

  /// Creates an [ErrorScreen] from an [ApplicationError].
  factory ErrorScreen.fromError(
    BuildContext context,
    ApplicationError error, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    Future<bool> maybePop() => Navigator.maybePop(context);
    final page = ErrorPage.fromError(context, error, onPrimaryActionPressed: maybePop, style: style);
    return ErrorScreen.fromPage(page, actions: style.associatedActions);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(title),
        automaticallyImplyLeading: false,
        actions: actions,
      ),
      body: ErrorPage(
        title: title,
        description: description,
        primaryButton: primaryButton,
        secondaryButton: secondaryButton,
        illustration: illustration,
      ),
    );
  }

  /// Navigates to the appropriate [ErrorScreen] for the given [ApplicationError].
  ///
  /// This method maps various [ApplicationError] types to their corresponding
  /// specialized screens. If no specialized screen is available for an error
  /// type, it defaults to a generic error screen and logs a warning.
  ///
  /// [style] determines the primary call-to-action button behavior (e.g., retry vs. close).
  /// [secured] specifies whether the screen should be pushed as a [SecuredPageRoute].
  static void show(
    BuildContext context,
    ApplicationError error, {
    ErrorCtaStyle style = ErrorCtaStyle.retry,
    bool secured = true,
  }) {
    final errorScreen = ErrorScreen.fromError(context, error, style: style);
    _showErrorScreen(context, secured: secured, errorScreen: errorScreen);
  }

  /// Internal helper to push the [ErrorScreen] onto the [Navigator] stack.
  static void _showErrorScreen(BuildContext context, {required ErrorScreen errorScreen, bool secured = true}) {
    final route = secured
        ? SecuredPageRoute(builder: (c) => errorScreen)
        : MaterialPageRoute(builder: (c) => errorScreen);
    Navigator.push(context, route);
  }
}

extension _ErrorScreenCtaStyleExtensions on ErrorCtaStyle {
  List<Widget> get associatedActions {
    return switch (this) {
      ErrorCtaStyle.retry => [],
      ErrorCtaStyle.close => [const CloseIconButton()],
    };
  }
}
