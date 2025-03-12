import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../forgot_pin/forgot_pin_screen.dart';
import 'argument/pin_timeout_screen_argument.dart';
import 'widget/pin_timeout_description.dart';

class PinTimeoutScreen extends StatelessWidget {
  static PinTimeoutScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return PinTimeoutScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError(
        'Make sure to pass in [PinTimeoutScreenArgument].toMap() when opening the PinTimeoutScreen',
      );
    }
  }

  final DateTime expiryTime;

  const PinTimeoutScreen({
    required this.expiryTime,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.pinTimeoutScreenHeadline,
                      scrollController: PrimaryScrollController.maybeOf(context),
                      actions: const [HelpIconButton()],
                    ),
                    SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: PinTimeoutDescription(
                          expiryTime: expiryTime,
                          onExpire: () => _onTimeoutExpired(context),
                        ),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                    const SliverPadding(
                      padding: EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: PageIllustration(
                          asset: WalletAssets.svg_blocked_temporary,
                          padding: EdgeInsets.zero,
                        ),
                      ),
                    ),
                    const SliverSizedBox(height: 24),
                  ],
                ),
              ),
            ),
            const Divider(),
            ConfirmButtons(
              forceVertical: !context.isLandscape,
              primaryButton: PrimaryButton(
                text: Text.rich(context.l10n.pinTimeoutScreenClearWalletCta.toTextSpan(context)),
                onPressed: () => ResetWalletDialog.show(context),
                icon: const Icon(Icons.delete_outline_rounded),
              ),
              secondaryButton: TertiaryButton(
                text: Text.rich(context.l10n.pinTimeoutScreenForgotPinCta.toTextSpan(context)),
                onPressed: () => ForgotPinScreen.show(context),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _onTimeoutExpired(BuildContext context) {
    // Avoid navigating if the timeout screen is not shown, this will
    // still be triggered if the user navigates back to this screen.
    if (ModalRoute.of(context)?.isCurrent != true) return;
    Navigator.pushNamedAndRemoveUntil(
      context,
      WalletRoutes.splashRoute,
      ModalRoute.withName(WalletRoutes.splashRoute),
    );
  }

  static void show(BuildContext context, DateTime expiryTime) {
    Navigator.restorablePushReplacementNamed(
      context,
      WalletRoutes.pinTimeoutRoute,
      arguments: PinTimeoutScreenArgument(expiryTime: expiryTime).toMap(),
    );
  }
}
