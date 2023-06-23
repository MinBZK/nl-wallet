import 'package:flutter/material.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/placeholder_screen.dart';
import '../menu/widget/menu_row.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.settingsScreenTitle),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    return ListView(
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
