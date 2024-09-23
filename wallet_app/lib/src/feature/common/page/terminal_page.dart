import 'dart:math';

import 'package:flutter/material.dart';

import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/button/primary_button.dart';
import '../widget/button/tertiary_button.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

class TerminalPage extends StatelessWidget {
  final String title;
  final String description;
  final String primaryButtonCta;
  final IconData primaryButtonIcon;
  final VoidCallback onPrimaryPressed;
  final String? secondaryButtonCta;
  final VoidCallback? onSecondaryButtonPressed;
  final Widget? illustration;

  bool get hasSecondaryButton => secondaryButtonCta != null;

  const TerminalPage({
    required this.title,
    required this.description,
    required this.primaryButtonCta,
    required this.onPrimaryPressed,
    this.primaryButtonIcon = Icons.arrow_forward_outlined,
    this.secondaryButtonCta,
    this.onSecondaryButtonPressed,
    this.illustration,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      bottom: false,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildScrollableSection(context),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Expanded(
      child: WalletScrollbar(
        child: ListView(
          padding: const EdgeInsets.only(top: 2, bottom: 24),
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TitleText(title),
                  const SizedBox(height: 8),
                  BodyText(description),
                ],
              ),
            ),
            illustration ?? _buildIllustrationPlaceHolder(context),
          ],
        ),
      ),
    );
  }

  Widget _buildIllustrationPlaceHolder(BuildContext context) {
    return Container(
      width: double.infinity,
      height: 200,
      decoration: BoxDecoration(
        color: context.colorScheme.primaryContainer,
        borderRadius: WalletTheme.kBorderRadius12,
      ),
      margin: const EdgeInsets.all(16),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Divider(height: 1),
        const SizedBox(height: 24),
        _buildPrimaryButton(context),
        SizedBox(height: hasSecondaryButton ? 16 : 0),
        if (hasSecondaryButton) _buildSecondaryButton(context),
        SizedBox(height: max(24, context.mediaQuery.viewPadding.bottom)),
      ],
    );
  }

  Widget _buildPrimaryButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: PrimaryButton(
        key: const Key('primaryButtonCta'),
        onPressed: onPrimaryPressed,
        text: Text.rich(primaryButtonCta.toTextSpan(context)),
        icon: Icon(primaryButtonIcon),
      ),
    );
  }

  Widget _buildSecondaryButton(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      width: double.infinity,
      child: TertiaryButton(
        key: const Key('secondaryButtonCta'),
        onPressed: onSecondaryButtonPressed,
        text: Text.rich(secondaryButtonCta!.toTextSpan(context)),
      ),
    );
  }
}
