import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';

import '../../domain/model/organization.dart';
import '../../domain/model/policy/policy.dart';
import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'policy_entries_builder.dart';
import 'policy_screen_arguments.dart';
import 'widget/policy_entry_row.dart';

class PolicyScreen extends StatelessWidget {
  static PolicyScreenArguments getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args! as PolicyScreenArguments;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args (type: ${args.runtimeType})', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a PolicyScreenArguments object');
    }
  }

  final Organization relyingParty;
  final Policy policy;
  final bool showSignatureRow;
  final VoidCallback? onReportIssuePressed;

  const PolicyScreen({
    required this.relyingParty,
    required this.policy,
    this.showSignatureRow = false,
    this.onReportIssuePressed,
    super.key,
  });

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
    final policyBuilder = PolicyEntriesBuilder(
      context,
      urlTheme,
      addSignatureEntry: showSignatureRow,
    );
    final entries = policyBuilder.build(relyingParty, policy);
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: context.l10n.policyScreenTitle,
            scrollController: PrimaryScrollController.maybeOf(context),
          ),
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text.rich(context.l10n.policyScreenSubtitle.toTextSpan(context)),
            ),
          ),
          const SliverSizedBox(height: 24),
          const SliverDivider(),
          SliverList.separated(
            itemBuilder: (context, index) => PolicyEntryRow.fromPolicyEntry(entries[index]),
            separatorBuilder: (context, i) => const Divider(),
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
    return ListButton(
      text: Text.rich(context.l10n.policyScreenReportIssueCta.toTextSpan(context)),
      onPressed: () {
        Navigator.pop(context);
        onReportIssuePressed?.call();
      },
    );
  }

  static void show(
    BuildContext context,
    Organization relyingParty,
    Policy policy, {
    VoidCallback? onReportIssuePressed,
  }) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => PolicyScreen(
          relyingParty: relyingParty,
          policy: policy,
          onReportIssuePressed: onReportIssuePressed,
        ),
      ),
    );
  }
}
