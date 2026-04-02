import 'package:flutter/material.dart';

import '../../domain/model/result/application_error.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/navigator_state_extension.dart';
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

  /// Creates a generic [ErrorScreen] for unspecified errors.
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  factory ErrorScreen.generic(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry}) {
    final page = ErrorPage.generic(context, style: style);
    return ErrorScreen.fromPage(page, actions: style.associatedActions);
  }

  /// Creates an [ErrorScreen] tailored for network-related errors.
  ///
  /// The screen content is determined by [error.hasInternet].
  factory ErrorScreen.network(
    BuildContext context, {
    required NetworkError error,
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    if (!error.hasInternet) {
      return ErrorScreen.noInternet(context, style: style);
    } else {
      return ErrorScreen.server(context, style: style);
    }
  }

  /// Creates an [ErrorScreen] specifically for 'no internet' scenarios.
  factory ErrorScreen.noInternet(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry}) {
    final page = ErrorPage.noInternet(context, style: style);
    return ErrorScreen.fromPage(page, actions: style.associatedActions);
  }

  /// Creates an [ErrorScreen] specifically for 'server error' scenarios.
  factory ErrorScreen.server(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry}) {
    final page = ErrorPage.server(context, style: style);
    return ErrorScreen.fromPage(page, actions: style.associatedActions);
  }

  /// Creates an [ErrorScreen] for when the device does not meet requirements.
  factory ErrorScreen.deviceIncompatible(BuildContext context) {
    final page = ErrorPage.deviceIncompatible(
      context,
      onPrimaryActionPressed: () => Navigator.of(context).resetToSplash(),
    );
    return ErrorScreen.fromPage(page, actions: const []);
  }

  /// Creates an [ErrorScreen] for session expiration scenarios.
  factory ErrorScreen.sessionExpired(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry}) {
    final page = ErrorPage.sessionExpired(context, style: style);
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

  /// Shows a generic [ErrorScreen] (e.g., 'something went wrong').
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  /// Set [secured] to false to bypass using a [SecuredPageRoute].
  static void showGeneric(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry, bool secured = true}) {
    final errorScreen = ErrorScreen.generic(context, style: style);
    _showErrorScreen(context, secured: secured, errorScreen: errorScreen);
  }

  /// Shows an [ErrorScreen] tailored for the given [NetworkError].
  ///
  /// The screen content is determined by the [error] state (e.g., server outage or no internet).
  /// Set [secured] to false to bypass using a [SecuredPageRoute].
  static void showNetwork(BuildContext context, {required NetworkError error, bool secured = true}) {
    final errorScreen = ErrorScreen.network(context, error: error);
    _showErrorScreen(context, secured: secured, errorScreen: errorScreen);
  }

  /// Shows an [ErrorScreen] for device incompatibility errors.
  ///
  /// Set [secured] to false to bypass using a [SecuredPageRoute].
  static void showDeviceIncompatible(BuildContext context, {bool secured = true}) {
    final errorScreen = ErrorScreen.deviceIncompatible(context);
    _showErrorScreen(context, secured: secured, errorScreen: errorScreen);
  }

  /// Shows an [ErrorScreen] for session expiration scenarios.
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  static void showSessionExpired(BuildContext context, {ErrorCtaStyle style = ErrorCtaStyle.retry}) {
    final errorScreen = ErrorScreen.sessionExpired(context, style: style);
    _showErrorScreen(context, errorScreen: errorScreen);
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
