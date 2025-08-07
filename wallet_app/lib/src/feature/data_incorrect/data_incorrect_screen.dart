import 'package:flutter/material.dart';

import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'widget/data_incorrect_option_row.dart';

class DataIncorrectScreen extends StatelessWidget {
  const DataIncorrectScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.dataIncorrectScreenHeaderTitle),
      ),
      body: SafeArea(
        child: WalletScrollbar(
          child: CustomScrollView(
            restorationId: 'data_incorrect',
            slivers: <Widget>[
              SliverToBoxAdapter(
                child: Padding(
                  padding: kDefaultTitlePadding,
                  child: TitleText(context.l10n.dataIncorrectScreenHeaderTitle),
                ),
              ),
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Text(
                    context.l10n.dataIncorrectScreenHeaderDescription,
                    style: context.textTheme.bodyLarge,
                  ),
                ),
              ),
              const SliverSizedBox(height: 32),
              const SliverToBoxAdapter(child: Divider()),
              SliverToBoxAdapter(
                child: DataIncorrectOptionRow(
                  title: context.l10n.dataIncorrectScreenDeclineTitle,
                  description: context.l10n.dataIncorrectScreenDeclineDescription,
                  cta: context.l10n.dataIncorrectScreenDeclineCta,
                  icon: Icons.block_flipped,
                  onTap: () => Navigator.pop(context, DataIncorrectResult.declineCard),
                ),
              ),
              const SliverToBoxAdapter(child: Divider()),
              SliverToBoxAdapter(
                child: DataIncorrectOptionRow(
                  title: context.l10n.dataIncorrectScreenApproveTitle,
                  description: context.l10n.dataIncorrectScreenApproveDescription,
                  cta: context.l10n.dataIncorrectScreenApproveCta,
                  icon: Icons.add,
                  onTap: () => Navigator.pop(context, DataIncorrectResult.acceptCard),
                ),
              ),
              const SliverToBoxAdapter(child: Divider()),
              const SliverSizedBox(height: 24),
              const SliverFillRemaining(
                hasScrollBody: false,
                fillOverscroll: true,
                child: BottomBackButton(),
              ),
            ],
          ),
        ),
      ),
    );
  }

  static Future<DataIncorrectResult?> show(BuildContext context) {
    return Navigator.of(context).push(
      SecuredPageRoute(
        builder: (context) => const DataIncorrectScreen(),
      ),
    );
  }
}

enum DataIncorrectResult { declineCard, acceptCard }
