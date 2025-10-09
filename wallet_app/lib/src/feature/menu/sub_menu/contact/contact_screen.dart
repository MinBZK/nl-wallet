import 'dart:async';

import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../common/mixin/lock_state_mixin.dart';
import '../../../common/screen/placeholder_screen.dart';
import '../../../common/widget/bullet_list.dart';
import '../../../common/widget/button/bottom_back_button.dart';
import '../../../common/widget/button/button_content.dart';
import '../../../common/widget/button/list_button.dart';
import '../../../common/widget/text/body_text.dart';
import '../../../common/widget/text/headline_small_text.dart';
import '../../../common/widget/text/title_text.dart';
import '../../../common/widget/wallet_app_bar.dart';
import '../../../common/widget/wallet_scrollbar.dart';
import '../../../dashboard/dashboard_screen.dart';

class ContactScreen extends StatefulWidget {
  const ContactScreen({super.key});

  @override
  State<ContactScreen> createState() => _ContactScreenState();
}

class _ContactScreenState extends State<ContactScreen> with LockStateMixin<ContactScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.contactScreenTitle),
      ),
      key: const Key('contactScreen'),
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
          padding: const EdgeInsets.only(left: 16, right: 16, top: 12, bottom: 24),
          child: Column(
            children: [
              TitleText(context.l10n.contactScreenTitle),
              const SizedBox(height: 8),
              BodyText(context.l10n.contactScreenDescription),
            ],
          ),
        ),
        const Divider(),
        Padding(
          padding: const EdgeInsets.only(left: 16, right: 16, top: 24),
          child: Column(
            children: [
              HeadlineSmallText(
                context.l10n.contactScreenCallUsTitle,
                style: BaseWalletTheme.headlineExtraSmallTextStyle,
              ),
              const SizedBox(height: 8),
              BodyText(context.l10n.contactScreenCallUsDescription),
              BulletList(items: context.l10n.contactScreenCallUsBullets.split('\n')),
            ],
          ),
        ),
        ListButton(
          text: Text.rich(context.l10n.contactScreenCallUsCta.toTextSpan(context)),
          icon: const Icon(Icons.phone_outlined),
          iconPosition: IconPosition.start,
          dividerSide: DividerSide.none,
          onPressed: () => PlaceholderScreen.showGeneric(context),
        ),
        const SizedBox(height: 24),
      ],
    );
  }

  @override
  FutureOr<void> onLock() {
    /// PVW-3104: Pop the MenuScreen if it's visible while the app gets locked
    if (ModalRoute.of(context)?.isCurrent ?? false) DashboardScreen.show(context);
  }

  @override
  FutureOr<void> onUnlock() {}
}
