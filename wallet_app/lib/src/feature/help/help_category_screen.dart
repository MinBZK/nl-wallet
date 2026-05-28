import 'package:flutter/material.dart';

import '../../domain/model/help/help_category.dart';
import '../../navigation/wallet_routes.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class HelpCategoryScreen extends StatelessWidget {
  final HelpCategory category;

  const HelpCategoryScreen({required this.category, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('helpCategoryScreen'),
      appBar: WalletAppBar(title: TitleText(category.title)),
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
        children: [
          Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(category.title),
          ),
          const SizedBox(height: 16),
          for (final sub in category.subcategories) ...[
            const Divider(),
            MenuItem(
              label: Text(sub.title),
              onPressed: () => Navigator.pushNamed(
                context,
                WalletRoutes.helpSubcategoryRoute,
                arguments: sub,
              ),
            ),
          ],
        ],
      ),
    );
  }
}
