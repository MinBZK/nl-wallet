import 'package:flutter/material.dart';

import '../../../../navigation/wallet_routes.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../../wallet_constants.dart';
import '../../../common/screen/placeholder_screen.dart';
import '../../../common/widget/button/bottom_back_button.dart';
import '../../../common/widget/menu_item.dart';
import '../../../common/widget/text/title_text.dart';
import '../../../common/widget/wallet_app_bar.dart';
import '../../../common/widget/wallet_scrollbar.dart';

class NeedHelpScreen extends StatelessWidget {
  const NeedHelpScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.needHelpScreenTitle),
      ),
      key: const Key('needHelpScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: _buildContentList(context),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContentList(BuildContext context) {
    return ListView(
      children: [
        Padding(
          padding: kDefaultTitlePadding,
          child: TitleText(context.l10n.needHelpScreenTitle),
        ),
        const SizedBox(height: 16),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenCardsTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenCardsSubtitle.toTextSpan(context)),
          leftIcon: const Icon(Icons.credit_card_outlined),
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenQrTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenQrSubtitle.toTextSpan(context)),
          leftIcon: const Icon(Icons.qr_code_outlined),
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenShareTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenShareSubtitle.toTextSpan(context)),
          leftIcon: Image.asset(WalletAssets.icon_disclose, color: context.theme.iconTheme.color),
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenActivitiesTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenActivitiesSubtitle.toTextSpan(context)),
          leftIcon: const Icon(Icons.history_outlined),
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenSecurityTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenSecuritySubtitle.toTextSpan(context)),
          leftIcon: const Icon(Icons.lock),
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenSettingsTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenSecuritySubtitle.toTextSpan(context)),
          leftIcon: const Icon(Icons.settings_outlined),
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const Divider(),
        MenuItem(
          label: Text.rich(context.l10n.needHelpScreenContactTitle.toTextSpan(context)),
          subtitle: Text.rich(context.l10n.needHelpScreenContactSubtitle.toTextSpan(context)),
          leftIcon: const Icon(Icons.headset_mic_outlined),
          onPressed: () => Navigator.pushNamed(context, WalletRoutes.contactRoute),
        ),
        const Divider(),
        const SizedBox(height: 24),
      ],
    );
  }
}
