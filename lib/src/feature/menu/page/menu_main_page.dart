import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../wallet_routes.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/version_text.dart';
import '../bloc/menu_bloc.dart';
import '../widget/menu_row.dart';

class MenuMainPage extends StatelessWidget {
  final String name;

  bool get showDesignSystemRow => kDebugMode;

  const MenuMainPage({Key? key, required this.name}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scrollbar(
      thumbVisibility: true,
      child: ListView(
        padding: const EdgeInsets.only(bottom: 24),
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
            child: Text(
              locale.menuMainPageGreeting(name),
              style: Theme.of(context).textTheme.headline2,
            ),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuMainPageHelpCta,
            icon: Icons.help_outline,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuMainPageHistoryCta,
            icon: Icons.history,
            onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuMainPageSettingsCta,
            icon: Icons.settings_outlined,
            onTap: () => context.read<MenuBloc>().add(MenuSettingsPressed()),
          ),
          const Divider(height: 1),
          MenuRow(
            label: locale.menuMainPageAboutCta,
            icon: Icons.info_outline,
            onTap: () => context.read<MenuBloc>().add(MenuAboutPressed()),
          ),
          const Divider(height: 1),
          if (showDesignSystemRow)
            MenuRow(
              label: locale.menuMainPageDesignCta,
              icon: Icons.design_services,
              onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
            ),
          if (showDesignSystemRow) const Divider(height: 1),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16.0, vertical: 32),
            child: VersionText(),
          ),
          Center(
            child: IntrinsicWidth(
              child: OutlinedButton(
                onPressed: () => context.read<MenuBloc>().add(MenuLockWalletPressed()),
                child: Row(
                  children: [
                    const Icon(Icons.lock, size: 14),
                    const SizedBox(width: 8),
                    Text(locale.menuMainPageLockCta),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
