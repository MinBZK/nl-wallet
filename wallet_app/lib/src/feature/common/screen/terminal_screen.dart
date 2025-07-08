import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/button/confirm/confirm_buttons.dart';
import '../widget/button/icon/help_icon_button.dart';
import '../widget/button/primary_button.dart';
import '../widget/page_illustration.dart';
import '../widget/sliver_wallet_app_bar.dart';
import '../widget/spacer/sliver_sized_box.dart';
import '../widget/text/body_text.dart';

class TerminalScreen extends StatelessWidget {
  final String title;
  final String description;
  final FitsWidthWidget primaryButton;
  final FitsWidthWidget? secondaryButton;
  final String illustration;

  const TerminalScreen({
    required this.title,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    required this.illustration,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: title,
            automaticallyImplyLeading: false,
            actions: const [HelpIconButton()],
          ),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: BodyText(description),
            ),
          ),
          const SliverSizedBox(height: 24),
          SliverToBoxAdapter(
            child: PageIllustration(asset: illustration),
          ),
          SliverFillRemaining(
            hasScrollBody: false,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                const Divider(),
                ConfirmButtons(
                  primaryButton: primaryButton,
                  secondaryButton: secondaryButton ?? const NeverFitsWidthWidget(child: SizedBox.shrink()),
                  hideSecondaryButton: secondaryButton == null,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

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
      primaryButton: config.primaryButton ??
          PrimaryButton(
            text: Text.rich(context.l10n.generalClose.toTextSpan(context)),
            icon: const Icon(Icons.close_outlined),
            onPressed: () => Navigator.maybePop(context),
          ),
      secondaryButton: config.secondaryButton,
    );
    final route = secured
        ? SecuredPageRoute(builder: (c) => terminalScreen)
        : MaterialPageRoute(
            builder: (c) => terminalScreen,
          );
    if (replaceCurrentRoute) {
      return Navigator.pushReplacement(context, route);
    } else {
      return Navigator.push(context, route);
    }
  }
}

/// Wrapper for all the info needed to render the [TerminalScreen].
class TerminalScreenConfig {
  final String title;
  final String description;
  final FitsWidthWidget? primaryButton;
  final FitsWidthWidget? secondaryButton;
  final String illustration;

  TerminalScreenConfig({
    required this.title,
    required this.description,
    this.primaryButton,
    this.secondaryButton,
    required this.illustration,
  });
}
