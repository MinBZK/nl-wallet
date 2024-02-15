import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/wallet_app_bar.dart';
import '../forgot_pin/forgot_pin_screen.dart';
import 'argument/pin_timeout_screen_argument.dart';
import 'widget/pin_timeout_description.dart';

class PinTimeoutScreen extends StatelessWidget {
  static PinTimeoutScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return PinTimeoutScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError(
          'Make sure to pass in [PinTimeoutScreenArgument].toMap() when opening the PinTimeoutScreen');
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
      appBar: WalletAppBar(
        title: Text(context.l10n.pinTimeoutScreenTitle),
        automaticallyImplyLeading: false,
        actions: [
          IconButton(
            onPressed: () => PlaceholderScreen.show(context, secured: false),
            icon: const Icon(Icons.info_outline_rounded),
          )
        ],
      ),
      body: PrimaryScrollController(
        controller: ScrollController(),
        child: Scrollbar(
          thumbVisibility: true,
          child: ListView(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            children: [
              Image.asset(
                WalletAssets.illustration_pin_timeout,
                width: double.infinity,
                fit: BoxFit.fitWidth,
              ),
              const SizedBox(height: 24),
              Text(
                context.l10n.pinTimeoutScreenHeadline,
                textAlign: TextAlign.start,
                style: context.textTheme.displayMedium,
              ),
              const SizedBox(height: 8),
              PinTimeoutDescription(
                expiryTime: expiryTime,
                onExpire: () => _onTimeoutExpired(context),
              ),
              const SizedBox(height: 24),
              Align(
                alignment: Alignment.centerLeft,
                child: LinkButton(
                  customPadding: EdgeInsets.zero,
                  child: Text(context.l10n.pinTimeoutScreenForgotPinCta),
                  onPressed: () => ForgotPinScreen.show(context),
                ),
              ),
            ],
          ),
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
