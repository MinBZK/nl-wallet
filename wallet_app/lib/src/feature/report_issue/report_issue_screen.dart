import 'package:flutter/material.dart';

import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/formatter/report_option_title_formatter.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/icon_row.dart';
import '../common/widget/wallet_app_bar.dart';

class ReportIssueScreen extends StatelessWidget {
  final List<ReportingOption> options;

  const ReportIssueScreen({required this.options, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: Text(context.l10n.reportIssueScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildContent(context),
            ),
            const BottomBackButton(showDivider: true),
          ],
        ),
      ),
    );
  }

  Scrollbar _buildContent(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        restorationId: 'data_incorrect',
        slivers: <Widget>[
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverList(delegate: _getOptionsDelegate(context)),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            context.l10n.reportIssueScreenHeaderTitle,
            style: context.textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getOptionsDelegate(BuildContext context) {
    return SliverChildBuilderDelegate(
      (context, index) => Column(
        children: [
          InkWell(
            onTap: () => Navigator.pop(context, options[index]),
            child: IconRow(
              icon: Icon(
                Icons.sms_failed_outlined,
                color: context.colorScheme.primary,
              ),
              text: Text(
                ReportOptionTitleFormatter.map(context, options[index]),
                style: context.textTheme.titleMedium,
              ),
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            ),
          ),
          const Divider(height: 1),
        ],
      ),
      childCount: options.length,
    );
  }

  static Future<ReportingOption?> show(BuildContext context, List<ReportingOption> options) {
    return Navigator.of(context).push(
      SecuredPageRoute(
        builder: (context) => ReportIssueScreen(options: options),
      ),
    );
  }
}

enum ReportingOption {
  unknownOrganization,
  requestNotInitiated,
  impersonatingOrganization,
  untrusted,
  overAskingOrganization,
  suspiciousOrganization,
  unreasonableTerms,
}
