import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../domain/model/result/application_error.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'error_button_builder.dart';

export 'error_cta_style.dart';

class ErrorPage extends StatelessWidget {
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

  const ErrorPage({
    required this.title,
    required this.description,
    required this.primaryButton,
    this.secondaryButton,
    this.illustration,
    super.key,
  });

  /// Creates an [ErrorPage] mapped from a specific [ApplicationError].
  ///
  /// This factory acts as a central dispatcher that returns the most appropriate
  /// [ErrorPage] variant for the given [error]. Note that not all errors are handled
  /// uniquely.
  ///
  /// Use [onPrimaryActionPressed] to handle the primary button action.
  /// Use [style] to determine the CTA button behavior (retry or close).
  factory ErrorPage.fromError(
    BuildContext context,
    ApplicationError error, {
    required VoidCallback onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    switch (error) {
      case GenericError():
        return ErrorPage.generic(context, onPrimaryActionPressed: onPrimaryActionPressed, style: style);
      case NetworkError(hasInternet: true):
        return ErrorPage.server(context, onPrimaryActionPressed: onPrimaryActionPressed, style: style);
      case NetworkError(hasInternet: false):
        return ErrorPage.noInternet(context, onPrimaryActionPressed: onPrimaryActionPressed, style: style);
      case SessionError():
        return ErrorPage.sessionExpired(context, onPrimaryActionPressed: onPrimaryActionPressed, style: style);
      case RelyingPartyError():
        return ErrorPage.relyingParty(context, onPrimaryActionPressed: onPrimaryActionPressed, style: style);
      default:
        Fimber.i('No specific handling defined for $error, defaulting to generic error page.');
        return ErrorPage.generic(context, onPrimaryActionPressed: onPrimaryActionPressed, style: style);
    }
  }

  /// Creates an [ErrorPage] for generic, unspecified errors.
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  factory ErrorPage.generic(
    BuildContext context, {
    VoidCallback? onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    return ErrorPage(
      title: context.l10n.errorScreenGenericHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenGenericDescriptionCloseVariant
          : context.l10n.errorScreenGenericDescription,
      illustration: WalletAssets.svg_error_general,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        style,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }

  /// Creates an [ErrorPage] for server-side errors or outages.
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  factory ErrorPage.server(
    BuildContext context, {
    VoidCallback? onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    return ErrorPage(
      title: context.l10n.errorScreenServerHeadline,
      description: style == ErrorCtaStyle.close
          ? context.l10n.errorScreenServerDescriptionCloseVariant
          : context.l10n.errorScreenServerDescription,
      illustration: WalletAssets.svg_error_server_outage,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        style,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }

  /// Creates an [ErrorPage] for when the device has no internet connection.
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  factory ErrorPage.noInternet(
    BuildContext context, {
    VoidCallback? onPrimaryActionPressed,
    required ErrorCtaStyle style,
  }) {
    return ErrorPage(
      title: context.l10n.errorScreenNoInternetHeadline,
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

  /// Creates an [ErrorPage] for when the device does not meet application requirements.
  factory ErrorPage.deviceIncompatible(
    BuildContext context, {
    required VoidCallback onPrimaryActionPressed,
  }) {
    return ErrorPage(
      title: context.l10n.errorScreenDeviceIncompatibleHeadline,
      description: context.l10n.errorScreenDeviceIncompatibleDescription,
      illustration: WalletAssets.svg_error_config_update,
      primaryButton: ErrorButtonBuilder.buildPrimaryButtonFor(
        context,
        ErrorCtaStyle.close,
        onPressed: onPrimaryActionPressed,
      ),
      secondaryButton: ErrorButtonBuilder.buildShowDetailsButton(context),
    );
  }

  /// Creates an [ErrorPage] for session expiration scenarios.
  ///
  /// Use [style] to determine the CTA button behavior (retry or close).
  /// [cta] can be used to override the default button text.
  factory ErrorPage.sessionExpired(
    BuildContext context, {
    VoidCallback? onPrimaryActionPressed,
    required ErrorCtaStyle style,
    String? cta,
  }) {
    return ErrorPage(
      title: context.l10n.errorScreenSessionExpiredHeadline,
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

  /// Creates an [ErrorPage] for when a session was cancelled externally.
  ///
  /// [organizationName] is the name of the organization with whom the session was cancelled.
  factory ErrorPage.cancelledSession(
    BuildContext context, {
    required String organizationName,
    VoidCallback? onPrimaryActionPressed,
  }) {
    return ErrorPage(
      title: context.l10n.errorScreenCancelledSessionHeadline,
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

  /// Creates an [ErrorPage] for errors related to a relying party (service provider).
  ///
  /// [organizationName] is the optional name of the organization where the error occurred.
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
      title: context.l10n.genericRelyingPartyErrorTitle,
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
                  SliverToBoxAdapter(
                    child: Padding(
                      padding: kDefaultTitlePadding,
                      child: TitleText(title),
                    ),
                  ),
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
