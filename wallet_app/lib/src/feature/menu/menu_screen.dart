import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../environment.dart';
import '../../data/service/announcement_service.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_constants.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/utility/do_on_init.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/menu_bloc.dart';

class MenuScreen extends StatelessWidget {
  final bool showDesignSystemRow;

  const MenuScreen({
    this.showDesignSystemRow = !kReleaseMode,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.menuScreenTitle),
        leading: const BackIconButton(),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          DoOnInit(
            onInit: (_) {
              final announcement = context.l10n.menuScreenWCAGPageAnnouncement(_buildMenuItems(context).length);
              context.read<AnnouncementService>().announce(announcement);
            },
          ),
          Expanded(
            child: WalletScrollbar(
              key: const Key('menuScreen'),
              child: _buildContentList(context),
            ),
          ),
          const BottomBackButton(),
        ],
      ),
    );
  }

  Widget _buildContentList(BuildContext context) {
    final menuItems = _buildMenuItems(context);

    // Opting to use a (non-sliver) ListView for better a11y support, [CustomScrollView] did not allow scrolling
    // to the 'logout' button for example. This might be fixable by using a combination of [SliverEnsureSemantics]
    // and CustomScrollViews with fixedScrollExtend, but seeing as this is a relatively short list, this seems like
    // the cleaner and (much) simpler solution. As fixedScrollExtend might also conflict with font scaling.
    return ListView(
      children: [
        Padding(
          padding: kDefaultTitlePadding,
          child: TitleText(context.l10n.menuScreenTitle),
        ),
        const SizedBox(height: 16),
        const Divider(),
        ListView.separated(
          physics: const NeverScrollableScrollPhysics(),
          itemBuilder: (c, i) => menuItems[i],
          separatorBuilder: (c, i) => const Divider(),
          itemCount: menuItems.length,
          shrinkWrap: true,
        ),
        const Divider(),
        const SizedBox(height: 40),
        Center(
          child: IntrinsicWidth(
            child: SecondaryButton(
              icon: const Icon(Icons.key_outlined),
              text: Text.rich(context.l10n.menuScreenLockCta.toTextSpan(context)),
              onPressed: () => context.read<MenuBloc>().add(MenuLockWalletPressed()),
            ),
          ),
        ),
        const SizedBox(height: 40),
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
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.needHelpRoute),
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
    if (showDesignSystemRow) {
      final designSystemItem = MenuItem(
        label: Text.rich(context.l10n.menuScreenDesignCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.design_services),
        onPressed: () => Navigator.restorablePushNamed(context, WalletRoutes.themeRoute),
      );
      defaultMenuItems.add(designSystemItem);
    }
    if (Environment.demoIndexUrl.isNotEmpty) {
      final browserTestItem = MenuItem(
        label: Text.rich(context.l10n.menuScreenBrowserCta.toTextSpan(context)),
        leftIcon: const Icon(Icons.web),
        onPressed: () => launchUrlString(Environment.demoIndexUrl, mode: LaunchMode.externalApplication),
        subtitle: Text('Open url: ${Environment.demoIndexUrl}'),
      );
      defaultMenuItems.add(browserTestItem);
    }
    return defaultMenuItems;
  }
}
