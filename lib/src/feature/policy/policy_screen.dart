import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/policy/interaction_policy.dart';
import '../common/widget/text_icon_button.dart';
import 'model/policy_entry.dart';
import 'policy_entries_builder.dart';
import 'widget/policy_entry_row.dart';

class PolicyScreen extends StatelessWidget {
  static InteractionPolicy getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as InteractionPolicy;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args (type: ${args.runtimeType})', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in an interaction policy.');
    }
  }

  final InteractionPolicy interactionPolicy;

  const PolicyScreen({required this.interactionPolicy, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'policy_scaffold',
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).policyScreenTitle),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    final urlTheme = Theme.of(context).textTheme.bodyText1!.copyWith(
          color: Theme.of(context).primaryColor,
          decoration: TextDecoration.underline,
        );
    final policyBuilder = PolicyEntriesBuilder(AppLocalizations.of(context), urlTheme);
    return Scrollbar(
      child: CustomScrollView(
        restorationId: 'policy_list',
        slivers: [
          SliverList(
            delegate: _getPolicyEntriesDelegate(policyBuilder.build(interactionPolicy)),
          ),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBackButton(context),
          )
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getPolicyEntriesDelegate(List<PolicyEntry> entries) {
    return SliverChildBuilderDelegate(
      (context, index) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: PolicyEntryRow(
              icon: entries[index].icon,
              title: Text.rich(entries[index].title),
              description: Text.rich(entries[index].description),
            ),
          ),
          const Divider(height: 1),
        ],
      ),
      childCount: entries.length,
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: SizedBox(
        height: 72,
        width: double.infinity,
        child: TextIconButton(
          onPressed: () => Navigator.pop(context),
          iconPosition: IconPosition.start,
          icon: Icons.arrow_back,
          child: Text(AppLocalizations.of(context).policyScreenBackCta),
        ),
      ),
    );
  }
}
