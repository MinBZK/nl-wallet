import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../domain/model/policy/policy.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_tile_button.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import 'policy_entries_builder.dart';
import 'widget/policy_entry_row.dart';

class PolicyScreen extends StatelessWidget {
  static Policy getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as Policy;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args (type: ${args.runtimeType})', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in an policy.');
    }
  }

  final Policy policy;
  final VoidCallback? onReportIssuePressed;

  const PolicyScreen({required this.policy, this.onReportIssuePressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'policy_scaffold',
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
    final urlTheme = context.textTheme.bodyLarge!.copyWith(
      color: context.colorScheme.primary,
      decoration: TextDecoration.underline,
    );
    final policyBuilder = PolicyEntriesBuilder(context, urlTheme);
    final entries = policyBuilder.build(policy);
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: context.l10n.policyScreenTitle,
          ),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(context.l10n.policyScreenSubtitle),
            ),
          ),
          const SliverSizedBox(height: 24),
          const SliverDivider(height: 1),
          SliverList.separated(
            itemBuilder: (context, index) {
              return PolicyEntryRow(
                icon: entries[index].icon,
                title: Text.rich(entries[index].title),
                description: Text.rich(entries[index].description),
              );
            },
            separatorBuilder: (context, i) => const Divider(height: 1),
            itemCount: entries.length,
          ),
          SliverToBoxAdapter(child: _buildReportIssueButton(context)),
          const SliverSizedBox(height: 24),
        ],
      ),
    );
  }

  Widget _buildReportIssueButton(BuildContext context) {
    if (onReportIssuePressed == null) return const SizedBox.shrink();
    return LinkTileButton(
      child: Text(context.l10n.policyScreenReportIssueCta),
      onPressed: () {
        Navigator.pop(context);
        onReportIssuePressed?.call();
      },
    );
  }

  static void show(BuildContext context, Policy policy, {VoidCallback? onReportIssuePressed}) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => PolicyScreen(
          policy: policy,
          onReportIssuePressed: onReportIssuePressed,
        ),
      ),
    );
  }
}
