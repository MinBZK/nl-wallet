import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../menu/widget/menu_row.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('settingsScreen'),
      body: Column(
        children: [
          Expanded(
            child: Scrollbar(
              child: CustomScrollView(
                slivers: [
                  SliverWalletAppBar(
                    title: context.l10n.settingsScreenTitle,
                  ),
                  _buildContentSliver(context),
                ],
              ),
            ),
          ),
          const BottomBackButton(),
        ],
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        const SizedBox(height: 16),
        MenuRow(
          label: context.l10n.settingsScreenChangePinCta,
          icon: Icons.key,
          onTap: () => PlaceholderScreen.show(context),
        ),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.settingsScreenSetupBiometricsCta,
          icon: Icons.fingerprint,
          onTap: () => PlaceholderScreen.show(context),
        ),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.settingsScreenChangeLanguageCta,
          icon: Icons.translate,
          onTap: () => Navigator.pushNamed(context, WalletRoutes.changeLanguageRoute),
        ),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.settingsScreenClearDataCta,
          icon: Icons.delete_outline,
          onTap: () => PlaceholderScreen.show(context),
        ),
        const Divider(height: 1),
      ],
    );
  }
}
