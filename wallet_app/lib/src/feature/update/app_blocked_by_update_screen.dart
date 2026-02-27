import 'dart:io';

import 'package:flutter/material.dart';
import 'package:store_redirect/store_redirect.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/launch_util.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/tertiary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/paragraphed_sliver_list.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

/// A screen displayed when the user is prevented from using the app because
/// the current version is no longer supported and a forced update is required.
class AppBlockedByUpdateScreen extends StatelessWidget {
  const AppBlockedByUpdateScreen({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.appBlockedByUpdateScreenTitle),
        actions: const [HelpIconButton()],
        automaticallyImplyLeading: true,
      ),
      body: SafeArea(
        child: WalletScrollbar(
          child: Column(
            children: [
              Expanded(
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: kDefaultTitlePadding,
                        child: TitleText(context.l10n.appBlockedByUpdateScreenTitle),
                      ),
                    ),
                    SliverPadding(
                      sliver: ParagraphedSliverList.splitContent(
                        Platform.isIOS
                            ? context.l10n.appBlockedByUpdateScreenDescriptioniOSVariant
                            : context.l10n.appBlockedByUpdateScreenDescription,
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
                  ],
                ),
              ),
              _buildBottomSection(context),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      children: [
        const Divider(),
        ConfirmButtons(
          forceVertical: !context.isLandscape,
          primaryButton: PrimaryButton(
            text: Text.rich(
              (Platform.isIOS ? context.l10n.generalToAppStoreCta : context.l10n.generalToPlayStoreCta).toTextSpan(
                context,
              ),
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
      ],
    );
  }
}
