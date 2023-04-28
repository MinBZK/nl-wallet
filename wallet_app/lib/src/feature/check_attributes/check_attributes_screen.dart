import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/attribute/data_attribute.dart';
import '../common/widget/attribute/attribute_row.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/placeholder_screen.dart';
import '../common/widget/sliver_sized_box.dart';

class CheckAttributesScreen extends StatelessWidget {
  final List<DataAttribute> attributes;

  const CheckAttributesScreen({required this.attributes, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.checkAttributesScreenTitle),
      ),
      body: Column(
        children: [
          Expanded(child: _buildContent(context)),
          const Divider(height: 1),
          const BottomBackButton(),
        ],
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        slivers: [
          const SliverSizedBox(height: 32),
          SliverList(
            delegate: SliverChildBuilderDelegate(
              (context, index) {
                final attr = attributes[index];
                return Padding(
                  padding: const EdgeInsets.only(left: 16, right: 16, bottom: 24),
                  child: AttributeRow(attribute: attr),
                );
              },
              childCount: attributes.length,
            ),
          ),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverSizedBox(height: 32),
        ],
      ),
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Divider(height: 1),
        LinkButton(
          onPressed: () => PlaceholderScreen.show(context),
          customPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Text(locale.checkAttributesScreenDataIncorrectCta),
        ),
        const Divider(height: 1),
      ],
    );
  }

  static void show(BuildContext context, List<DataAttribute> attributes) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => CheckAttributesScreen(attributes: attributes)),
    );
  }
}
