import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/text_icon_button.dart';
import 'bloc/verifier_policy_bloc.dart';
import 'model/policy_entry.dart';
import 'policy_entries_builder.dart';
import 'widget/policy_entry_row.dart';

class VerifierPolicyScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args (type: ${args.runtimeType})', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a verification sessionId.');
    }
  }

  const VerifierPolicyScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'verifier_policy_scaffold',
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).verifierPolicyScreenTitle),
      ),
      body: BlocBuilder<VerifierPolicyBloc, VerifierPolicyState>(
        builder: (context, state) {
          if (state is VerifierPolicyInitial) return const CenteredLoadingIndicator();
          if (state is VerifierPolicyLoadInProgress) return const CenteredLoadingIndicator();
          if (state is VerifierPolicyLoadFailure) return _buildError(context, state);
          if (state is VerifierPolicyLoadSuccess) return _buildLoaded(context, state);
          throw UnsupportedError('Unknown state: $state');
        },
      ),
    );
  }

  Widget _buildError(BuildContext context, VerifierPolicyLoadFailure state) {
    final locale = AppLocalizations.of(context);
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Text(locale.verifierPolicyScreenErrorDescription),
          ElevatedButton(
            onPressed: () => context.read<VerifierPolicyBloc>().add(VerifierPolicyLoadTriggered(state.sessionId)),
            child: Text(locale.verifierPolicyScreenRetryCta),
          ),
        ],
      ),
    );
  }

  Widget _buildLoaded(BuildContext context, VerifierPolicyLoadSuccess state) {
    final urlTheme = Theme.of(context).textTheme.bodyText1!.copyWith(
          color: Theme.of(context).primaryColor,
          decoration: TextDecoration.underline,
        );
    final policyBuilder = PolicyEntriesBuilder(AppLocalizations.of(context), urlTheme);
    return CustomScrollView(
      restorationId: 'verifier_policy_list',
      slivers: [
        SliverList(
          delegate: _getPolicyEntriesDelegate(policyBuilder.build(state.policy)),
        ),
        SliverFillRemaining(
          hasScrollBody: false,
          fillOverscroll: true,
          child: _buildBackButton(context),
        )
      ],
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
          child: Text(AppLocalizations.of(context).verifierPolicyScreenBackCta),
        ),
      ),
    );
  }
}
