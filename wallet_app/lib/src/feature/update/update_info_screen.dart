import 'dart:io';

import 'package:flutter/material.dart';
import 'package:store_redirect/store_redirect.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../common/page/page_illustration.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/close_icon_button.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/paragraphed_sliver_list.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class UpdateInfoScreen extends StatelessWidget {
  const UpdateInfoScreen({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: WalletScrollbar(
          child: Column(
            children: [
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.updateInfoScreenTitle,
                      actions: const [HelpIconButton(), CloseIconButton()],
                      scrollController: PrimaryScrollController.maybeOf(context),
                      automaticallyImplyLeading: true,
                    ),
                    SliverPadding(
                      sliver: ParagraphedSliverList.splitContent(
                        Platform.isIOS
                            ? context.l10n.updateInfoScreenDescriptioniOSVariant
                            : context.l10n.updateInfoScreenDescription,
                      ),
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                    ),
                    const SliverToBoxAdapter(
                      child: Padding(
                        padding: EdgeInsets.symmetric(vertical: 24),
                        child: PageIllustration(
                          asset: WalletAssets.svg_update_app,
                        ),
                      ),
                    ),
                    _buildButtons(context),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildButtons(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Align(
        alignment: Alignment.bottomCenter,
        child: ConfirmButtons(
          primaryButton: PrimaryButton(
            text: Text(Platform.isIOS ? context.l10n.generalToAppStoreCta : context.l10n.generalToPlayStoreCta),
            icon: const Icon(Icons.north_east_outlined),
            onPressed: () => StoreRedirect.redirect,
          ),
          secondaryButton: TertiaryButton(
            text: Text(context.l10n.generalClose),
            icon: const Icon(Icons.close_outlined),
            onPressed: () => Navigator.maybePop(context),
          ),
        ),
      ),
    );
  }
}
