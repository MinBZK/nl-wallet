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
import '../../wallet_constants.dart';
import '../common/mixin/lock_state_mixin.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/spacer/sliver_divider.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/do_on_init.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/menu_bloc.dart';

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
      appBar: WalletAppBar(
        title: TitleText(context.l10n.menuScreenTitle),
        leading: const BackIconButton(),
      ),
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
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: kDefaultTitlePadding,
                        child: TitleText(context.l10n.menuScreenTitle),
                      ),
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
      MenuItem(
        label: Text.rich(context.l10n.menuScreenTourCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.play_arrow),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.tourOverviewRoute),
      ),
      MenuItem(
        label: Text.rich(context.l10n.menuScreenHelpCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.help_outline),
        onPressed: () => PlaceholderScreen.showHelp(context),
      ),
      MenuItem(
        label: Text.rich(context.l10n.menuScreenScanQrCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.qr_code_rounded),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.qrRoute),
      ),
      MenuItem(
        label: Text.rich(context.l10n.menuScreenHistoryCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.history),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.walletHistoryRoute),
      ),
      MenuItem(
        label: Text.rich(context.l10n.menuScreenSettingsCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.settings_outlined),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.settingsRoute),
      ),
      MenuItem(
        label: Text.rich(context.l10n.menuScreenFeedbackCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.comment_outlined),
        onPressed: () => PlaceholderScreen.showGeneric(context),
      ),
      MenuItem(
        label: Text.rich(context.l10n.menuScreenAboutCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.info_outline),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.aboutRoute),
      ),
    ];
    if (widget.showDesignSystemRow) {
      final designSystemItem = MenuItem(
        label: Text.rich(context.l10n.menuScreenDesignCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.design_services),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
      );
      defaultMenuItems.add(designSystemItem);
    }
    if (Environment.demoRelyingPartyUrl.isNotEmpty) {
      final browserTestItem = MenuItem(
        label: Text.rich(context.l10n.menuScreenBrowserCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.web),
        onPressed: () => launchUrlString(Environment.demoRelyingPartyUrl, mode: LaunchMode.externalApplication),
        subtitle: Text('Open url: ${Environment.demoRelyingPartyUrl}'),
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
