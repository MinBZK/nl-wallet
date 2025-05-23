import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/sheet/error_details_sheet.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_scrollbar.dart';
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
        text: Text.rich(context.l10n.generalShowDetailsCta.toTextSpan(context)),
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
        text: Text.rich(context.l10n.generalShowDetailsCta.toTextSpan(context)),
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

  factory ErrorPage.sessionExpired(
    BuildContext context, {
    VoidCallback? onPrimaryActionPressed,
    required ErrorCtaStyle style,
    String? cta,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenSessionExpiredHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenSessionExpiredDescriptionCloseVariant
          : context.l10n.errorScreenSessionExpiredDescription,
      illustration: WalletAssets.svg_error_session_expired,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        style,
        onPressed: onPrimaryActionPressed,
        cta: cta,
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }

  factory ErrorPage.cancelledSession(
    BuildContext context, {
    required String organizationName,
    VoidCallback? onPrimaryActionPressed,
  }) {
    return ErrorPage(
      headline: context.l10n.errorScreenCancelledSessionHeadline,
      description: context.l10n.errorScreenCancelledSessionDescription(organizationName),
      illustration: WalletAssets.svg_stopped,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        ErrorCtaStyle.close,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }

  factory ErrorPage.relyingParty(
    BuildContext context, {
    String? organizationName,
    VoidCallback? onPrimaryActionPressed,
    ErrorCtaStyle style = ErrorCtaStyle.retry,
  }) {
    final description = organizationName == null
        ? context.l10n.genericRelyingPartyErrorDescription
        : context.l10n.genericRelyingPartyErrorDescriptionWithOrganizationName(organizationName);
    return ErrorPage(
      headline: context.l10n.genericRelyingPartyErrorTitle,
      description: description,
      illustration: WalletAssets.svg_error_card_blocked,
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
      child: WalletScrollbar(
        child: Column(
          children: [
            Expanded(
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
                ],
              ),
            ),
            _buildBottomSection(context),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        children: [
          const Divider(),
          ConfirmButtons(
            forceVertical: !context.isLandscape,
            flipVertical: true,
            hideSecondaryButton: secondaryButton == null,
            secondaryButton: secondaryButton ?? const TertiaryButton(text: Text('' /* invisible placeholder */)),
            primaryButton: primaryButton,
          ),
        ],
      ),
    );
  }
}
