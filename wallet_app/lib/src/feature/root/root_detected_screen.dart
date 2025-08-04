import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/button_content.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/paragraphed_sliver_list.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class RootDetectedScreen extends StatelessWidget {
  const RootDetectedScreen({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.rootDetectedScreenTitle),
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
                        child: TitleText(context.l10n.rootDetectedScreenTitle),
                      ),
                    ),
                    SliverPadding(
                      sliver: ParagraphedSliverList.splitContent(context.l10n.rootDetectedScreenDescription),
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                    ),
                    const SliverToBoxAdapter(
                      child: Padding(
                        padding: EdgeInsets.symmetric(vertical: 24),
                        child: PageIllustration(
                          asset: WalletAssets.svg_error_rooted,
                        ),
                      ),
                    ),
                    if (Platform.isAndroid) _buildCloseAppButton(context),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCloseAppButton(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          const Divider(),
          Padding(
            padding: const EdgeInsets.all(16),
            child: PrimaryButton(
              onPressed: () => SystemChannels.platform.invokeMethod<void>('SystemNavigator.pop', true),
              icon: const Icon(Icons.close_outlined),
              iconPosition: IconPosition.start,
              mainAxisAlignment: MainAxisAlignment.center,
              text: Text.rich(context.l10n.generalClose.toTextSpan(context)),
            ),
          ),
        ],
      ),
    );
  }
}
