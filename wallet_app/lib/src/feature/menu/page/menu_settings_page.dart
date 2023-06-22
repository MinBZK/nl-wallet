import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/placeholder_screen.dart';
import '../bloc/menu_bloc.dart';
import '../widget/menu_row.dart';

class MenuSettingsPage extends StatelessWidget {
  const MenuSettingsPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return WillPopScope(
      onWillPop: () async {
        context.read<MenuBloc>().add(MenuBackPressed());
        return false;
      },
      child: ListView(
        children: [
          const SizedBox(height: 16),
          MenuRow(
            label: context.l10n.menuSettingsPageChangePinCta,
            icon: Icons.key,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuSettingsPageSetupBiometricsCta,
            icon: Icons.fingerprint,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuSettingsPageChangeLanguageCta,
            icon: Icons.translate,
            onTap: () => Navigator.pushNamed(context, WalletRoutes.changeLanguageRoute),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuSettingsPageClearDataCta,
            icon: Icons.delete_outline,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
