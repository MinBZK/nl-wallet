import 'dart:math';

import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/sheet/help_sheet.dart';
import '../common/widget/button/text_icon_button.dart';
import '../common/widget/sliver_sized_box.dart';

class ErrorPage extends StatelessWidget {
  final String? illustration;
  final String headline;
  final String description;
  final String primaryActionText;
  final IconData? primaryActionIcon;
  final String? secondaryActionText;
  final VoidCallback onPrimaryActionPressed;
  final VoidCallback? onSecondaryActionPressed;

  const ErrorPage({
    required this.headline,
    required this.description,
    required this.primaryActionText,
    this.primaryActionIcon,
    required this.onPrimaryActionPressed,
    this.illustration,
    this.secondaryActionText,
    this.onSecondaryActionPressed,
    super.key,
  });

  factory ErrorPage.generic(
    BuildContext context, {
    String? headline,
    String? description,
    String? primaryActionText,
    IconData? primaryActionIcon,
    VoidCallback? onPrimaryActionPressed,
    String? secondaryActionText,
    VoidCallback? onSecondaryActionPressed,
  }) {
    bool hasSecondaryAction = onSecondaryActionPressed != null;
    return ErrorPage(
      headline: headline ?? context.l10n.errorScreenGenericHeadline,
      description: description ?? context.l10n.errorScreenGenericDescription,
      illustration: WalletAssets.illustration_general_error,
      primaryActionText: primaryActionText ?? context.l10n.errorScreenGenericCloseCta,
      primaryActionIcon: primaryActionIcon,
      onPrimaryActionPressed: onPrimaryActionPressed ?? () => Navigator.pop(context),
      secondaryActionText: hasSecondaryAction ? (secondaryActionText ?? context.l10n.errorScreenGeneralHelpCta) : null,
      onSecondaryActionPressed: onSecondaryActionPressed,
    );
  }

  factory ErrorPage.network(
    BuildContext context, {
    String? primaryActionText,
    IconData? primaryActionIcon,
    VoidCallback? onPrimaryActionPressed,
    bool showHelpSheetAsSecondaryCta = true,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenServerHeadline,
      description: context.l10n.errorScreenServerDescription,
      illustration: WalletAssets.illustration_server_error,
      primaryActionText: primaryActionText ?? context.l10n.errorScreenServerCloseCta,
      primaryActionIcon: primaryActionIcon,
      onPrimaryActionPressed: onPrimaryActionPressed ?? () => Navigator.pop(context),
      secondaryActionText: showHelpSheetAsSecondaryCta ? context.l10n.errorScreenServerHelpCta : null,
      onSecondaryActionPressed: showHelpSheetAsSecondaryCta ? () => HelpSheet.show(context) : null,
    );
  }

  factory ErrorPage.noInternet(
    BuildContext context, {
    String? primaryActionText,
    IconData? primaryActionIcon,
    VoidCallback? onPrimaryActionPressed,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenNoInternetHeadline,
      description: context.l10n.errorScreenNoInternetDescription,
      illustration: WalletAssets.illustration_no_internet_error,
      primaryActionText: primaryActionText ?? context.l10n.generalRetry,
      primaryActionIcon: primaryActionIcon,
      onPrimaryActionPressed: onPrimaryActionPressed ?? () => Navigator.pop(context),
    );
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      bottom: false, // handled by _buildBottomSection
      child: Scrollbar(
        thumbVisibility: true,
        trackVisibility: true,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: CustomScrollView(
            slivers: [
              const SliverSizedBox(height: 24),
              SliverToBoxAdapter(
                child: Text(
                  headline,
                  textAlign: TextAlign.start,
                  style: context.textTheme.displayMedium,
                ),
              ),
              const SliverSizedBox(height: 8),
              SliverToBoxAdapter(
                child: Text(
                  description,
                  textAlign: TextAlign.start,
                  style: context.textTheme.bodyLarge,
                ),
              ),
              const SliverSizedBox(height: 24),
              SliverToBoxAdapter(
                child: _buildIllustration(),
              ),
              const SliverSizedBox(height: 24),
              SliverFillRemaining(
                hasScrollBody: false,
                fillOverscroll: true,
                child: _buildBottomSection(context),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildIllustration() {
    if (illustration == null) {
      return Container(
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: const Color(0xFFF5F5FD),
          borderRadius: BorderRadius.circular(8),
        ),
        width: double.infinity,
        height: 100,
        child: const Text('Placeholder image'),
      );
    } else {
      return Image.asset(
        illustration!,
        fit: BoxFit.fitWidth,
      );
    }
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        _buildPrimaryButton(),
        if (secondaryActionText != null) ...[
          const SizedBox(height: 8),
          Center(
            child: TextIconButton(
              onPressed: onSecondaryActionPressed,
              child: Text(secondaryActionText!),
            ),
          ),
        ],
        SizedBox(height: max(24, context.mediaQuery.viewPadding.bottom)),
      ],
    );
  }

  Widget _buildPrimaryButton() {
    return ElevatedButton(
      onPressed: onPrimaryActionPressed,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (primaryActionIcon != null) Icon(primaryActionIcon!, size: 16),
          if (primaryActionIcon != null) const SizedBox(width: 12),
          Text(primaryActionText),
        ],
      ),
    );
  }
}
