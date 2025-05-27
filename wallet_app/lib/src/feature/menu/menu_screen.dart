import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../environment.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/mixin/lock_state_mixin.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/utility/do_on_init.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/menu_bloc.dart';
import 'widget/menu_row.dart';

class MenuScreen extends StatefulWidget {
  final bool showDesignSystemRow;

  const MenuScreen({
    this.showDesignSystemRow = !kReleaseMode,
    super.key,
  });

  @override
  State<MenuScreen> createState() => _MenuScreenState();
}

class _MenuScreenState extends State<MenuScreen> with LockStateMixin<MenuScreen> {
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
              child: WalletScrollbar(
                key: const Key('menuScreen'),
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.menuScreenTitle,
                      scrollController: PrimaryScrollController.maybeOf(context),
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
      itemBuilder: (context, i) => menuItems[i],
      itemCount: menuItems.length,
      separatorBuilder: (context, i) => const Divider(),
    );
    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 16),
        const SliverDivider(),
        itemsList,
        const SliverDivider(),
        const SliverSizedBox(height: 40),
        SliverToBoxAdapter(
          child: Center(
            child: IntrinsicWidth(
              child: SecondaryButton(
                icon: const Icon(Icons.key_outlined),
                text: Text.rich(context.l10n.menuScreenLockCta.toTextSpan(context)),
                onPressed: () => context.read<MenuBloc>().add(MenuLockWalletPressed()),
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
        label: context.l10n.menuScreenTourCta,
        icon: Icons.play_arrow,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.tourRoute),
      ),
      MenuRow(
        label: context.l10n.menuScreenHelpCta,
        icon: Icons.help_outline,
        onTap: () => PlaceholderScreen.showHelp(context),
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
        onTap: () => PlaceholderScreen.showGeneric(context),
      ),
      MenuRow(
        label: context.l10n.menuScreenAboutCta,
        icon: Icons.info_outline,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.aboutRoute),
      ),
    ];
    if (widget.showDesignSystemRow) {
      final designSystemItem = MenuRow(
        label: context.l10n.menuScreenDesignCta,
        icon: Icons.design_services,
        onTap: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
      );
      defaultMenuItems.add(designSystemItem);
    }
    if (Environment.demoRelyingPartyUrl.isNotEmpty) {
      final browserTestItem = MenuRow(
        label: context.l10n.menuScreenBrowserCta,
        subtitle: 'Open url: ${Environment.demoRelyingPartyUrl}',
        icon: Icons.web,
        onTap: () => launchUrlString(Environment.demoRelyingPartyUrl, mode: LaunchMode.externalApplication),
      );
      defaultMenuItems.add(browserTestItem);
    }
    return defaultMenuItems;
  }

  @override
  void onLock() {
    /// PVW-3104: Pop the MenuScreen if it's visible while the app gets locked
    if (ModalRoute.of(context)?.isCurrent ?? false) {
      Navigator.maybePop(context);
    }
  }

  @override
  FutureOr<void> onUnlock() {
    /* unused */
  }
}
