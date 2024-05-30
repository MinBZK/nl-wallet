import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/page/page_illustration.dart';
import '../common/sheet/error_details_sheet.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import 'error_button_builder.dart';

export 'error_cta_style.dart';

class ErrorPage extends StatelessWidget {
  final String? illustration;
  final String headline;
  final String description;
  final FitsWidthWidget primaryButton;
  final FitsWidthWidget? secondaryButton;

  const ErrorPage({
    required this.headline,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    this.illustration,
    super.key,
  });

  factory ErrorPage.generic(
    BuildContext context, {
    required VoidCallback onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenGenericHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenGenericDescriptionCloseVariant
          : context.l10n.errorScreenGenericDescription,
      illustration: WalletAssets.svg_error_general,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        style,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.generalShowDetailsCta),
        icon: const Icon(Icons.help_outline_rounded),
        onPressed: () => ErrorDetailsSheet.show(context),
      ),
    );
  }

  factory ErrorPage.network(
    BuildContext context, {
    required VoidCallback onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenServerHeadline,
      description: context.l10n.errorScreenServerDescription,
      illustration: WalletAssets.svg_error_server_outage,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        style,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: TertiaryButton(
        text: Text(context.l10n.generalShowDetailsCta),
        icon: const Icon(Icons.help_outline_rounded),
        onPressed: () => ErrorDetailsSheet.show(context),
      ),
    );
  }

  factory ErrorPage.noInternet(
    BuildContext context, {
    required VoidCallback onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenNoInternetHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenNoInternetDescriptionCloseVariant
          : context.l10n.errorScreenNoInternetDescription,
      illustration: WalletAssets.svg_error_no_internet,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        style,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Scrollbar(
        child: CustomScrollView(
          slivers: [
            const SliverSizedBox(height: 24),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: TitleText(headline),
              ),
            ),
            const SliverSizedBox(height: 8),
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
}
