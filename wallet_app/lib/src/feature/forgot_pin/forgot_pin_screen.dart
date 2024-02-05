import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/wallet_app_bar.dart';

class ForgotPinScreen extends StatelessWidget {
  const ForgotPinScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('forgotPinScreen'),
      appBar: WalletAppBar(
        title: Text(context.l10n.forgotPinScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(child: _buildScrollableSection(context)),
            _buildBottomSection(context),
          ],
        ),
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Scrollbar(
      child: ListView(
        padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
        children: [
          Image.asset(WalletAssets.illustration_forgot_pin_header, fit: BoxFit.fitWidth),
          const SizedBox(height: 24),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                context.l10n.forgotPinScreenHeadline,
                textAlign: TextAlign.start,
                style: context.textTheme.displayMedium,
              ),
              const SizedBox(height: 8),
              Text(
                context.l10n.forgotPinScreenDescription,
                textAlign: TextAlign.start,
                style: context.textTheme.bodyLarge,
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: ElevatedButton(
            onPressed: () async {
              final navigator = Navigator.of(context);
              await context.read<ResetWalletUseCase>().invoke();
              navigator.restorablePushNamedAndRemoveUntil(
                WalletRoutes.setupSecurityRoute,
                ModalRoute.withName(WalletRoutes.splashRoute),
              );
            },
            child: Text(context.l10n.forgotPinScreenCta),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const ForgotPinScreen()),
    );
  }
}
