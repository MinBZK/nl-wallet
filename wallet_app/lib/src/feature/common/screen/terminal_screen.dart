import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/button/confirm/confirm_buttons.dart';
import '../widget/button/icon/help_icon_button.dart';
import '../widget/button/primary_button.dart';
import '../widget/page_illustration.dart';
import '../widget/paragraphed_list.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_app_bar.dart';
import '../widget/wallet_scrollbar.dart';

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
      appBar: WalletAppBar(
        title: TitleText(title),
        automaticallyImplyLeading: false,
        actions: const [HelpIconButton()],
      ),
      body: SafeArea(
        bottom: false,
        child: _buildScrollableSection(),
      ),
    );
  }

  LayoutBuilder _buildScrollableSection() {
    return LayoutBuilder(
      builder: (context, constraints) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: ConstrainedBox(
              constraints: BoxConstraints(minHeight: constraints.maxHeight),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  _buildContentSection(context),
                  _buildBottomSection(context),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildContentSection(BuildContext context) {
    return Column(
      children: [
        const SizedBox(height: 12),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TitleText(title),
              const SizedBox(height: 8),
              ParagraphedList.splitContent(description),
            ],
          ),
        ),
        const SizedBox(height: 24),
        PageIllustration(asset: illustration),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final hideSecondaryButton = secondaryButton == null;
    return Column(
      children: [
        const Divider(),
        ConfirmButtons(
          primaryButton: primaryButton,
          secondaryButton: hideSecondaryButton
              ? const PrimaryButton(text: Text('')) /* always hidden */
              : secondaryButton!,
          hideSecondaryButton: hideSecondaryButton,
        ),
      ],
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
      primaryButton:
          config.primaryButton ??
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
