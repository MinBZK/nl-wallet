import 'package:flutter/material.dart';

import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/paragraphed_sliver_list.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';

class CardDataIncorrectScreen extends StatelessWidget {
  const CardDataIncorrectScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.cardDataIncorrectScreenSubhead),
        actions: const [HelpIconButton()],
      ),
      key: const Key('cardDataIncorrectScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverToBoxAdapter(
                      child: Padding(
                        padding: kDefaultTitlePadding,
                        child: TitleText(context.l10n.cardDataIncorrectScreenSubhead),
                      ),
                    ),
                    SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: ParagraphedSliverList.splitContent(
                        context.l10n.cardDataIncorrectScreenDescription,
                      ),
                    ),
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

  static void show(BuildContext context) {
    Navigator.push(
      context,
      SecuredPageRoute(builder: (c) => const CardDataIncorrectScreen()),
    );
  }
}
