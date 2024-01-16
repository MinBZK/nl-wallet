import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import 'bloc/menu_bloc.dart';
import 'widget/menu_row.dart';

class MenuScreen extends StatelessWidget {
  bool get showDesignSystemRow => kDebugMode;

  const MenuScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      key: const Key('menuScreen'),
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            leading: const SizedBox.shrink(),
            title: context.l10n.menuScreenTitle,
          ),
          _buildContentSliver(),
        ],
      ),
    );
  }

  Widget _buildContentSliver() {
    return BlocBuilder<MenuBloc, MenuState>(
      builder: (context, MenuState state) {
        return switch (state) {
          MenuInitial() => _buildLoadingSliver(),
          MenuLoadInProgress() => _buildLoadingSliver(),
          MenuLoadSuccess() => _buildSuccessSliver(context, state),
        };
      },
    );
  }

  Widget _buildLoadingSliver() {
    return const SliverFillRemaining(
      child: CenteredLoadingIndicator(),
    );
  }

  Widget _buildSuccessSliver(BuildContext context, MenuLoadSuccess state) {
    return SliverList.list(
      children: [
        const SizedBox(height: 16),
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
          label: context.l10n.menuScreenFeedbackCta,
          icon: Icons.comment_outlined,
          onTap: () => PlaceholderScreen.show(context),
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
        const SizedBox(height: 40),
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
        const SizedBox(height: 16),
      ],
    );
  }
}
