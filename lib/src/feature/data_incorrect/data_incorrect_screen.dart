import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/text_icon_button.dart';
import 'widget/data_incorrect_option_row.dart';

class DataIncorrectScreen extends StatelessWidget {
  const DataIncorrectScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.dataIncorrectScreenTitle),
      ),
      body: Scrollbar(
        child: CustomScrollView(
          restorationId: 'data_incorrect',
          slivers: <Widget>[
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverToBoxAdapter(child: Divider(height: 1)),
            SliverToBoxAdapter(
              child: DataIncorrectOptionRow(
                title: locale.dataIncorrectScreenDeclineTitle,
                description: locale.dataIncorrectScreenDeclineDescription,
                cta: locale.dataIncorrectScreenDeclineCta,
                onTap: () => Navigator.pop(context, DataIncorrectResult.declineCard),
              ),
            ),
            const SliverToBoxAdapter(child: Divider(height: 1)),
            SliverToBoxAdapter(
              child: DataIncorrectOptionRow(
                title: locale.dataIncorrectScreenApproveTitle,
                description: locale.dataIncorrectScreenApproveDescription,
                cta: locale.dataIncorrectScreenApproveCta,
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
            top: BorderSide(width: 0.5, color: Theme.of(context).dividerColor),
          ),
        ),
        height: 72,
        width: double.infinity,
        child: TextIconButton(
          onPressed: () => Navigator.pop(context, null),
          iconPosition: IconPosition.start,
          icon: Icons.arrow_back,
          child: Text(AppLocalizations.of(context).dataIncorrectScreenBackCta),
        ),
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.dataIncorrectScreenHeaderTitle,
            style: Theme.of(context).textTheme.headline2,
          ),
          const SizedBox(height: 16),
          Text(
            locale.dataIncorrectScreenHeaderDescription,
            style: Theme.of(context).textTheme.bodyText1,
          ),
        ],
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
