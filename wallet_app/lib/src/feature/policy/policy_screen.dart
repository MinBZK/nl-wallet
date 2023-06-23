import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../domain/model/policy/policy.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_button.dart';
import 'model/policy_entry.dart';
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
      appBar: AppBar(
        title: Text(context.l10n.policyScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildBody(context),
            ),
            const BottomBackButton(
              showDivider: true,
            ),
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
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          SliverList(
            delegate: _getPolicyEntriesDelegate(policyBuilder.build(policy)),
          ),
          SliverToBoxAdapter(child: _buildReportIssueButton(context)),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getPolicyEntriesDelegate(List<PolicyEntry> entries) {
    return SliverChildBuilderDelegate(
      (context, index) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          PolicyEntryRow(
            icon: entries[index].icon,
            title: Text.rich(entries[index].title),
            description: Text.rich(entries[index].description),
          ),
          const Divider(height: 1),
        ],
      ),
      childCount: entries.length,
    );
  }

  Widget _buildReportIssueButton(BuildContext context) {
    if (onReportIssuePressed == null) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Divider(height: 1),
        LinkButton(
          onPressed: () {
            Navigator.pop(context);
            onReportIssuePressed?.call();
          },
          customPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Text(context.l10n.policyScreenReportIssueCta),
        ),
        const Divider(height: 1),
      ],
    );
  }

  static void show(BuildContext context, Policy policy, {VoidCallback? onReportIssuePressed}) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (c) => PolicyScreen(
          policy: policy,
          onReportIssuePressed: onReportIssuePressed,
        ),
      ),
    );
  }
}
