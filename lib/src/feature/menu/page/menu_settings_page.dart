import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/placeholder_screen.dart';
import '../bloc/menu_bloc.dart';
import '../widget/menu_row.dart';

class MenuSettingsPage extends StatelessWidget {
  const MenuSettingsPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return WillPopScope(
      onWillPop: () async {
        context.read<MenuBloc>().add(MenuBackPressed());
        return false;
      },
      child: ListView(
        children: [
          const SizedBox(height: 16),
          MenuRow(
            label: locale.menuSettingsPageChangePinCta,
            icon: Icons.key,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuSettingsPageSetupBiometricsCta,
            icon: Icons.fingerprint,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuSettingsPageChangeLanguageCta,
            icon: Icons.translate,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuSettingsPageClearDataCta,
            icon: Icons.delete_outline,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
        ],
      ),
    );
  }
}
