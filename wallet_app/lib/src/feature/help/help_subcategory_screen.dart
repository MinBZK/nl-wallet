import 'package:flutter/material.dart';

import '../../domain/model/help/help_subcategory.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'argument/help_topic_screen_argument.dart';

class HelpSubcategoryScreen extends StatelessWidget {
  final HelpSubcategory subcategory;

  const HelpSubcategoryScreen({required this.subcategory, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('helpSubcategoryScreen'),
      appBar: WalletAppBar(title: TitleText(subcategory.title)),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildBody(context),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return WalletScrollbar(
      child: ListView(
        scrollCacheExtent: context.screenReaderListCacheExtent,
        children: [
          Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(subcategory.title),
          ),
          const SizedBox(height: 16),
          for (final topic in subcategory.topics) ...[
            const Divider(),
            MenuItem(
              label: Text(topic.title),
              onPressed: () => _onTopicTap(context, topic.id),
            ),
          ],
        ],
      ),
    );
  }

  void _onTopicTap(BuildContext context, String topicId) {
    Navigator.pushNamed(
      context,
      WalletRoutes.helpTopicRoute,
      arguments: HelpTopicScreenArgument(topicId: topicId),
    );
  }
}
