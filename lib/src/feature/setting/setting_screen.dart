import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_routes.dart';
import '../common/widget/version_text.dart';

class SettingScreen extends StatelessWidget {
  const SettingScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).settingsScreenTitle),
      ),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('Placeholder; settings'),
            _buildHistoryButton(context),
            _buildThemeButton(context),
            const VersionText(),
          ],
        ),
      ),
    );
  }

  Widget _buildHistoryButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: ElevatedButton(
        child: const Text('Gebruikersgeschiedenis'),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
      ),
    );
  }

  Widget _buildThemeButton(BuildContext context) {
    if (kDebugMode) {
      return Padding(
        padding: const EdgeInsets.all(16),
        child: ElevatedButton(
          child: const Text('Design System'),
          onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
        ),
      );
    } else {
      return const SizedBox.shrink();
    }
  }
}
