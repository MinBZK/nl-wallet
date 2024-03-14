import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/dialog/reset_wallet_dialog.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/sliver_wallet_app_bar.dart';

class ForgotPinScreen extends StatelessWidget {
  const ForgotPinScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('forgotPinScreen'),
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
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(title: context.l10n.forgotPinScreenTitle),
          SliverPadding(
            padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
            sliver: _buildContentSliver(context),
          ),
        ],
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        Text(
          context.l10n.forgotPinScreenDescription,
          textAlign: TextAlign.start,
          style: context.textTheme.bodyLarge,
        ),
        const SizedBox(height: 32),
        Image.asset(WalletAssets.illustration_forgot_pin_header, fit: BoxFit.fitWidth),
      ],
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      children: [
        const Divider(height: 1),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
          child: Column(
            children: [
              PrimaryButton(
                onPressed: () => ResetWalletDialog.show(context),
                text: context.l10n.forgotPinScreenCta,
              ),
              const SizedBox(height: 12),
              SecondaryButton(
                onPressed: () => Navigator.maybePop(context),
                text: context.l10n.generalBottomBackCta,
                icon: Icons.arrow_back,
              ),
            ],
          ),
        ),
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
