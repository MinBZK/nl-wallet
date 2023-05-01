import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../util/mapper/reporting_option_title_mapper.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/icon_row.dart';
import '../common/widget/sliver_sized_box.dart';

class ReportIssueScreen extends StatelessWidget {
  final List<ReportingOption> options;

  const ReportIssueScreen({required this.options, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.reportIssueScreenTitle),
      ),
      body: Column(
        children: [
          Expanded(child: _buildContent(context)),
          const BottomBackButton(showDivider: true),
        ],
      ),
    );
  }

  Scrollbar _buildContent(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        restorationId: 'data_incorrect',
        slivers: <Widget>[
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverList(delegate: _getOptionsDelegate(context)),
          const SliverSizedBox(height: 32),
        ],
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
            locale.reportIssueScreenHeaderTitle,
            style: Theme.of(context).textTheme.displayMedium,
          ),
          const SizedBox(height: 16),
          Text(
            locale.reportIssueScreenHeaderSubtitle,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getOptionsDelegate(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return SliverChildBuilderDelegate(
      (context, index) => Column(
        children: [
          InkWell(
            onTap: () => Navigator.pop(context, options[index]),
            child: IconRow(
              icon: const Icon(Icons.gpp_maybe_outlined),
              text: Text(ReportingOptionTitleMapper.map(locale, options[index])),
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
    return Navigator.of(context).push(MaterialPageRoute(builder: (context) {
      return ReportIssueScreen(options: options);
    }));
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
