import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/text_icon_button.dart';
import 'widget/data_incorrect_option_row.dart';

class DataIncorrectScreen extends StatelessWidget {
  const DataIncorrectScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.dataIncorrectScreenTitle),
      ),
      body: Scrollbar(
        child: CustomScrollView(
          restorationId: 'data_incorrect',
          slivers: <Widget>[
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverToBoxAdapter(child: Divider(height: 1)),
            SliverToBoxAdapter(
              child: DataIncorrectOptionRow(
                title: context.l10n.dataIncorrectScreenDeclineTitle,
                description: context.l10n.dataIncorrectScreenDeclineDescription,
                cta: context.l10n.dataIncorrectScreenDeclineCta,
                icon: Icons.block_flipped,
                onTap: () => Navigator.pop(context, DataIncorrectResult.declineCard),
              ),
            ),
            const SliverToBoxAdapter(child: Divider(height: 1)),
            SliverToBoxAdapter(
              child: DataIncorrectOptionRow(
                title: context.l10n.dataIncorrectScreenApproveTitle,
                description: context.l10n.dataIncorrectScreenApproveDescription,
                cta: context.l10n.dataIncorrectScreenApproveCta,
                icon: Icons.add,
                onTap: () => Navigator.pop(context, DataIncorrectResult.acceptCard),
              ),
            ),
            const SliverToBoxAdapter(child: Divider(height: 1)),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: _buildBackButton(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Container(
        decoration: BoxDecoration(
          border: Border(
            top: BorderSide(width: 0.5, color: context.colorScheme.outlineVariant),
          ),
        ),
        height: 72,
        width: double.infinity,
        child: TextIconButton(
          onPressed: () => Navigator.pop(context, null),
          iconPosition: IconPosition.start,
          icon: Icons.arrow_back,
          child: Text(context.l10n.dataIncorrectScreenBackCta),
        ),
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: MergeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              context.l10n.dataIncorrectScreenHeaderTitle,
              style: context.textTheme.displayMedium,
            ),
            const SizedBox(height: 16),
            Text(
              context.l10n.dataIncorrectScreenHeaderDescription,
              style: context.textTheme.bodyLarge,
            ),
          ],
        ),
      ),
    );
  }

  static Future<DataIncorrectResult?> show(BuildContext context) {
    return Navigator.of(context).push(MaterialPageRoute(builder: (context) {
      return const DataIncorrectScreen();
    }));
  }
}

enum DataIncorrectResult { declineCard, acceptCard }
