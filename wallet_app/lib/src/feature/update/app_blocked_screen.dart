import 'dart:io';

import 'package:flutter/material.dart';
import 'package:store_redirect/store_redirect.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/launch_util.dart';
import '../../wallet_assets.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/paragraphed_sliver_list.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class AppBlockedScreen extends StatelessWidget {
  const AppBlockedScreen({
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
                      title: context.l10n.appBlockedScreenTitle,
                      actions: const [HelpIconButton()],
                      scrollController: PrimaryScrollController.maybeOf(context),
                      automaticallyImplyLeading: true,
                    ),
                    SliverPadding(
                      sliver: ParagraphedSliverList.splitContent(
                        Platform.isIOS
                            ? context.l10n.appBlockedScreenDescriptioniOSVariant
                            : context.l10n.appBlockedScreenDescription,
                      ),
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                    ),
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(vertical: 24),
                        child: PageIllustration(
                          asset: Platform.isIOS ? WalletAssets.svg_update_app_ios : WalletAssets.svg_update_app,
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
            text: Text.rich(
              (Platform.isIOS ? context.l10n.generalToAppStoreCta : context.l10n.generalToPlayStoreCta)
                  .toTextSpan(context),
            ),
            icon: const Icon(Icons.north_east_outlined),
            onPressed: () => StoreRedirect.redirect,
          ),
          secondaryButton: TertiaryButton(
            text: Text.rich(context.l10n.generalNeedHelpCta.toTextSpan(context)),
            icon: const Icon(Icons.help_outline_rounded),
            onPressed: () => launchUrlStringCatching('https://edi.pleio.nl/'),
          ),
        ),
      ),
    );
  }
}
