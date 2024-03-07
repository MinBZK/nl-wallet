import 'dart:math';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../widget/button/primary_button.dart';
import '../widget/button/text_icon_button.dart';

class TerminalPage extends StatelessWidget {
  final String title;
  final String description;
  final String primaryButtonCta;
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
          const SizedBox(height: 16),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Expanded(
      child: Scrollbar(
        child: ListView(
          padding: const EdgeInsets.symmetric(vertical: 24),
          children: [
            Padding(
              padding: const EdgeInsets.all(16),
              child: MergeSemantics(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: context.textTheme.displayMedium,
                      textAlign: TextAlign.start,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      description,
                      style: context.textTheme.bodyLarge,
                      textAlign: TextAlign.start,
                    )
                  ],
                ),
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
        borderRadius: BorderRadius.circular(12),
      ),
      margin: const EdgeInsets.all(16),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildPrimaryButton(),
        SizedBox(height: hasSecondaryButton ? 16 : 0),
        if (hasSecondaryButton) _buildSecondaryButton(),
        SizedBox(height: max(24, context.mediaQuery.viewPadding.bottom)),
      ],
    );
  }

  Widget _buildPrimaryButton() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: PrimaryButton(
        key: const Key('primaryButtonCta'),
        onPressed: onPrimaryPressed,
        text: primaryButtonCta,
      ),
    );
  }

  Widget _buildSecondaryButton() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      width: double.infinity,
      child: TextIconButton(
        key: const Key('secondaryButtonCta'),
        onPressed: onSecondaryButtonPressed,
        iconPosition: IconPosition.start,
        centerChild: false,
        child: Text(secondaryButtonCta!),
      ),
    );
  }
}
