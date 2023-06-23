import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/placeholder_screen.dart';
import '../common/widget/version_text.dart';
import 'bloc/menu_bloc.dart';
import 'widget/menu_row.dart';

class MenuScreen extends StatelessWidget {
  bool get showDesignSystemRow => kDebugMode;

  const MenuScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.menuScreenTitle),
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    return BlocBuilder<MenuBloc, MenuState>(
      builder: (context, state) {
        if (state is MenuInitial) return const CenteredLoadingIndicator();
        if (state is MenuLoadInProgress) return const CenteredLoadingIndicator();
        if (state is MenuLoadSuccess) return _buildSuccess(context, state);
        throw UnsupportedError('Unknown state: ${state.runtimeType}');
      },
    );
  }

  Widget _buildSuccess(BuildContext context, MenuLoadSuccess state) {
    return Scrollbar(
      child: ListView(
        padding: const EdgeInsets.only(bottom: 24),
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
            child: Text(
              context.l10n.menuScreenGreeting(state.name),
              style: context.textTheme.displayMedium,
            ),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuScreenHelpCta,
            icon: Icons.help_outline,
            onTap: () => PlaceholderScreen.show(context),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuScreenHistoryCta,
            icon: Icons.history,
            onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuScreenSettingsCta,
            icon: Icons.settings_outlined,
            onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.settingsRoute),
          ),
          const Divider(height: 1),
          MenuRow(
            label: context.l10n.menuScreenAboutCta,
            icon: Icons.info_outline,
            onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.aboutRoute),
          ),
          const Divider(height: 1),
          if (showDesignSystemRow)
            MenuRow(
              label: context.l10n.menuScreenDesignCta,
              icon: Icons.design_services,
              onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
            ),
          if (showDesignSystemRow) const Divider(height: 1),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16, vertical: 32),
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
                    Text(context.l10n.menuScreenLockCta),
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
