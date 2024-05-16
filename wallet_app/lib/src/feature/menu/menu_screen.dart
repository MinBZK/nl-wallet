import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/utility/do_on_init.dart';
import 'bloc/menu_bloc.dart';
import 'widget/menu_row.dart';

class MenuScreen extends StatelessWidget {
  bool get showDesignSystemRow => kDebugMode;

  const MenuScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            DoOnInit(
              onInit: (_) {
                SemanticsService.announce(
                  context.l10n.menuScreenWCAGPageAnnouncement(_buildMenuItems(context).length),
                  TextDirection.ltr,
                );
              },
            ),
            Expanded(
              child: Scrollbar(
                key: const Key('menuScreen'),
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.menuScreenTitle,
                      leading: const BackIconButton(),
                    ),
                    _buildContentSliver(context),
                  ],
                ),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    final menuItems = _buildMenuItems(context);
    final itemsList = SliverList.separated(
      itemBuilder: (c, i) => menuItems[i],
      separatorBuilder: (c, i) => const Divider(height: 1),
      itemCount: menuItems.length,
    );
    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 16),
        const SliverDivider(height: 1),
        itemsList,
        const SliverDivider(height: 1),
        const SliverSizedBox(height: 40),
        SliverToBoxAdapter(
          child: Center(
            child: IntrinsicWidth(
              child: OutlinedButton(
                onPressed: () async {
                  context.read<MenuBloc>().add(MenuLockWalletPressed());
                  Navigator.popUntil(context, ModalRoute.withName(WalletRoutes.dashboardRoute));
                },
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
        ),
        const SliverSizedBox(height: 40),
      ],
    );
  }

  List<Widget> _buildMenuItems(BuildContext context) {
    final defaultMenuItems = [
      MenuRow(
        label: context.l10n.menuScreenHelpCta,
        icon: Icons.help_outline,
        onTap: () => PlaceholderScreen.show(context),
      ),
      MenuRow(
        label: context.l10n.menuScreenScanQrCta,
        icon: Icons.qr_code_rounded,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.qrRoute),
      ),
      MenuRow(
        label: context.l10n.menuScreenHistoryCta,
        icon: Icons.history,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
      ),
      MenuRow(
        label: context.l10n.menuScreenSettingsCta,
        icon: Icons.settings_outlined,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.settingsRoute),
      ),
      MenuRow(
        label: context.l10n.menuScreenFeedbackCta,
        icon: Icons.comment_outlined,
        onTap: () => PlaceholderScreen.show(context),
      ),
      MenuRow(
        label: context.l10n.menuScreenAboutCta,
        icon: Icons.info_outline,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.aboutRoute),
      ),
    ];
    if (showDesignSystemRow) {
      var designSystemItem = MenuRow(
        label: context.l10n.menuScreenDesignCta,
        icon: Icons.design_services,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
      );
      return defaultMenuItems..add(designSystemItem);
    }
    return defaultMenuItems;
  }
}
