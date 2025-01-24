import 'package:flutter/material.dart';

import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/formatter/report_option_title_formatter.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/info_row.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

class ReportIssueScreen extends StatelessWidget {
  final List<ReportingOption> options;

  const ReportIssueScreen({required this.options, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildContent(context),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          SliverWalletAppBar(
            title: context.l10n.reportIssueScreenTitle,
            scrollController: PrimaryScrollController.maybeOf(context),
          ),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 24),
          const SliverToBoxAdapter(child: Divider()),
          _buildOptionsSliver(context),
          const SliverToBoxAdapter(child: Divider()),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
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

  Widget _buildOptionsSliver(BuildContext context) {
    return SliverList.separated(
      itemBuilder: (c, i) {
        return InfoRow(
          icon: Icons.history,
          title: Text(ReportOptionTitleFormatter.map(context, options[i])),
          onTap: () => PlaceholderScreen.showGeneric(context),
        );
      },
      separatorBuilder: (c, i) => const Divider(),
      itemCount: options.length,
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
