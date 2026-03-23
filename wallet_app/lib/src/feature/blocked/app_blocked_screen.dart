import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/navigator_state_extension.dart';
import '../../wallet_assets.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/screen/terminal_screen.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import 'argument/app_blocked_screen_argument.dart';
import 'bloc/app_blocked_bloc.dart';

/// Screen displayed when the app/wallet has been blocked or revoked.
///
/// This screen provides information to the user about why they cannot access their wallet
/// and offers next steps, such as contacting the helpdesk or creating a new wallet
/// (if permitted).
class AppBlockedScreen extends StatelessWidget {
  /// Extracts [AppBlockedScreenArgument] from [RouteSettings].
  static AppBlockedScreenArgument? getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return AppBlockedScreenArgument.fromJson(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      return null;
    }
  }

  const AppBlockedScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AppBlockedBloc, AppBlockedState>(
      builder: (context, state) {
        return switch (state) {
          AppBlockedInitial() => _buildLoading(context),
          AppBlockedError() => _buildError(context),
          AppBlockedByAdmin() => _buildBlockedByAdmin(context, state.canRegisterNewAccount),
          AppBlockedByUser() => _buildBlockedByUser(context),
          AppBlockedSolutionCompromised() => _buildSolutionCompromised(context),
        };
      },
    );
  }

  Widget _buildLoading(BuildContext context) {
    return Scaffold(
      body: ColoredBox(
        color: context.colorScheme.onSurface.withValues(alpha: 0.2),
        child: const CenteredLoadingIndicator(showCircularBackground: true),
      ),
    );
  }

  Widget _buildError(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: Text(context.l10n.errorScreenGenericHeadline),
      ),
      body: ErrorPage.generic(
        context,
        onPrimaryActionPressed: () => Navigator.of(context).resetToSplash(),
        style: .close,
      ),
    );
  }

  /// UI for when the user intentionally revoked the wallet.
  Widget _buildBlockedByUser(BuildContext context) {
    return TerminalScreen(
      title: context.l10n.appBlockedScreenByUserTitle,
      description: context.l10n.appBlockedScreenByUserDescription,
      primaryButton: _buildHelpdeskButton(context),
      illustration: WalletAssets.svg_move_source_success,
    );
  }

  /// UI for when the wallet was blocked by the administrative side.
  ///
  /// [canRegisterNewAccount] determines if the "Create new wallet" button is shown.
  Widget _buildBlockedByAdmin(BuildContext context, bool canRegisterNewAccount) {
    final title = canRegisterNewAccount
        ? context.l10n.appBlockedScreenTitle
        : context.l10n.appBlockedScreenPermanentTitle;
    final description = canRegisterNewAccount
        ? context.l10n.appBlockedScreenDescription
        : context.l10n.appBlockedScreenPermanentDescription;
    return TerminalScreen(
      title: title,
      description: description,
      primaryButton: canRegisterNewAccount ? _buildCreateWalletButton(context) : _buildHelpdeskButton(context),
      secondaryButton: canRegisterNewAccount ? _buildHelpdeskButton(context) : null,
      illustration: WalletAssets.svg_blocked_final,
    );
  }

  /// UI for when the wallet solution is revoked/compromised.
  Widget _buildSolutionCompromised(BuildContext context) {
    return TerminalScreen(
      title: context.l10n.appBlockedScreenSolutionRevokedTitle,
      description: context.l10n.appBlockedScreenSolutionRevokedDescription,
      primaryButton: _buildMoreInfoButton(context),
      illustration: WalletAssets.svg_blocked_final,
    );
  }

  FitsWidthWidget _buildCreateWalletButton(BuildContext context) => SecondaryButton(
    text: Text(context.l10n.appBlockedScreenNewWalletCta),
    onPressed: () async {
      final navigator = Navigator.of(context);
      await context.read<ResetWalletUseCase>().invoke();
      unawaited(navigator.resetToSplash());
    },
  );

  FitsWidthWidget _buildHelpdeskButton(BuildContext context) => TertiaryButton(
    text: Text(context.l10n.appBlockedScreenHelpdeskCta),
    onPressed: () => Navigator.of(context).pushNamed(WalletRoutes.contactRoute),
  );

  FitsWidthWidget _buildMoreInfoButton(BuildContext context) => TertiaryButton(
    text: Text(context.l10n.appBlockedScreenMoreInfoCta),
    icon: const Icon(Icons.north_east_outlined),
    onPressed: () => PlaceholderScreen.showGeneric(context, secured: false),
  );
}
