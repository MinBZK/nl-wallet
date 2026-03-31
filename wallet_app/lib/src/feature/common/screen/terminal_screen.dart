import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../page/terminal_page.dart';
import '../widget/button/confirm/confirm_buttons.dart';
import '../widget/button/icon/help_icon_button.dart';
import '../widget/button/primary_button.dart';
import '../widget/page_illustration.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_app_bar.dart';

/// A screen that displays a [TerminalPage] within a [Scaffold].
///
/// It includes a [WalletAppBar] with a [HelpIconButton].
/// This is typically used as the final screen in a flow to show status
/// (success, error, etc.) and provide next steps.
class TerminalScreen extends StatelessWidget {
  /// The title displayed in the app bar and at the top of the page.
  final String title;

  /// The description text displayed below the title.
  ///
  /// The text is split into paragraphs and displayed using [ParagraphedList].
  final String description;

  /// The primary action button.
  ///
  /// Usually a [PrimaryButton].
  final FitsWidthWidget primaryButton;

  /// The optional secondary action button.
  ///
  /// Usually a [TertiaryButton].
  final FitsWidthWidget? secondaryButton;

  /// The asset path for the illustration.
  ///
  /// This is used to create a [PageIllustration] displayed in the center.
  final String illustration;

  /// Whether buttons should be laid out vertically when both are present.
  ///
  /// This is passed to the underlying [TerminalPage] and is ignored in landscape mode.
  final bool preferVerticalButtonLayout;

  const TerminalScreen({
    required this.title,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    required this.illustration,
    this.preferVerticalButtonLayout = false,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(title),
        automaticallyImplyLeading: false,
        actions: const [HelpIconButton()],
      ),
      body: TerminalPage(
        title: title,
        description: description,
        primaryButton: primaryButton,
        secondaryButton: secondaryButton,
        illustration: PageIllustration(asset: illustration),
        preferVerticalButtonLayout: preferVerticalButtonLayout,
      ),
    );
  }

  /// Navigates to a [TerminalScreen] using the provided [config].
  ///
  /// [secured] determines if a [SecuredPageRoute] (true) or [MaterialPageRoute] (false)
  /// should be used. Defaults to true.
  ///
  /// [replaceCurrentRoute] if true, uses [Navigator.pushReplacement] instead of [Navigator.push].
  static Future<void> show(
    BuildContext context, {
    required TerminalScreenConfig config,
    bool secured = true,
    bool replaceCurrentRoute = false,
  }) {
    final terminalScreen = TerminalScreen(
      title: config.title,
      description: config.description,
      illustration: config.illustration,
      primaryButton:
          config.primaryButton ??
          PrimaryButton(
            text: Text.rich(context.l10n.generalClose.toTextSpan(context)),
            icon: const Icon(Icons.close_outlined),
            onPressed: () => Navigator.maybePop(context),
          ),
      secondaryButton: config.secondaryButton,
      preferVerticalButtonLayout: config.preferVerticalButtonLayout,
    );
    final route = secured
        ? SecuredPageRoute(builder: (c) => terminalScreen)
        : MaterialPageRoute(builder: (c) => terminalScreen);
    if (replaceCurrentRoute) {
      return Navigator.pushReplacement(context, route);
    } else {
      return Navigator.push(context, route);
    }
  }
}

/// Configuration data for rendering and navigating to a [TerminalScreen].
class TerminalScreenConfig {
  /// The title of the screen.
  final String title;

  /// The description text of the screen.
  final String description;

  /// The primary action button.
  ///
  /// If null, a default close button is used when calling [TerminalScreen.show].
  final FitsWidthWidget? primaryButton;

  /// The secondary action button.
  final FitsWidthWidget? secondaryButton;

  /// The asset path for the illustration.
  final String illustration;

  /// Whether buttons should be laid out vertically when both are present.
  ///
  /// This is ignored in landscape mode.
  final bool preferVerticalButtonLayout;

  TerminalScreenConfig({
    required this.title,
    required this.description,
    this.primaryButton,
    this.secondaryButton,
    required this.illustration,
    this.preferVerticalButtonLayout = false,
  });
}
