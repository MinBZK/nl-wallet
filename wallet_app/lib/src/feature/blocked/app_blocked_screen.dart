import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/navigator_state_extension.dart';
import '../../wallet_assets.dart';
import '../common/screen/terminal_screen.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/wallet_app_bar.dart';
import '../error/error_page.dart';
import 'argument/app_blocked_screen_argument.dart';
import 'bloc/app_blocked_bloc.dart';

class AppBlockedScreen extends StatelessWidget {
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

  Widget _buildBlockedByUser(BuildContext context) {
    return TerminalScreen(
      title: context.l10n.appBlockedScreenByUserTitle,
      description: context.l10n.appBlockedScreenByUserDescription,
      primaryButton: _buildHelpdeskButton(context),
      illustration: WalletAssets.svg_move_source_success,
    );
  }

  Widget _buildBlockedByAdmin(BuildContext context, bool canRegisterNewAccount) {
    return TerminalScreen(
      title: context.l10n.appBlockedScreenTitle,
      description: context.l10n.appBlockedScreenDescription,
      primaryButton: canRegisterNewAccount ? _buildCreateWalletButton(context) : _buildHelpdeskButton(context),
      secondaryButton: canRegisterNewAccount ? _buildHelpdeskButton(context) : null,
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
}
