import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import 'argument/history_detail_screen_argument.dart';
import 'bloc/history_detail_bloc.dart';
import 'widget/page/history_detail_disclose_page.dart';
import 'widget/page/history_detail_issue_page.dart';
import 'widget/page/history_detail_login_page.dart';
import 'widget/page/history_detail_sign_page.dart';

class HistoryDetailScreen extends StatelessWidget {
  static HistoryDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return HistoryDetailScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [HistoryDetailScreenArgument] when opening the HistoryDetailScreen');
    }
  }

  const HistoryDetailScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('historyDetailScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: BlocBuilder<HistoryDetailBloc, HistoryDetailState>(
                builder: (context, state) {
                  return switch (state) {
                    HistoryDetailInitial() => _buildLoading(context),
                    HistoryDetailLoadInProgress() => _buildLoading(context),
                    HistoryDetailLoadSuccess() => PrimaryScrollController(
                        controller: ScrollController(),
                        child: _buildSuccess(context, state),
                      ),
                    HistoryDetailLoadFailure() => _buildError(context),
                  };
                },
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildLoading(BuildContext context) {
    return CustomScrollView(
      slivers: [
        SliverWalletAppBar(
          title: context.l10n.historyDetailScreenTitle,
          scrollController: PrimaryScrollController.maybeOf(context),
        ),
        const SliverFillRemaining(
          hasScrollBody: false,
          child: CenteredLoadingIndicator(),
        ),
      ],
    );
  }

  Widget _buildSuccess(BuildContext context, HistoryDetailLoadSuccess state) {
    final WalletEvent event = state.event;
    switch (event) {
      case DisclosureEvent():
        switch (event.type) {
          case DisclosureType.regular:
            return HistoryDetailDisclosePage(event: event);
          case DisclosureType.login:
            return HistoryDetailLoginPage(event: event);
        }
      case IssuanceEvent():
        return HistoryDetailIssuePage(event: event);
      case SignEvent():
        return HistoryDetailSignPage(event: event);
    }
  }

  Widget _buildError(BuildContext context) {
    return CustomScrollView(
      slivers: [
        SliverWalletAppBar(
          title: context.l10n.historyDetailScreenTitle,
          scrollController: PrimaryScrollController.maybeOf(context),
        ),
        SliverFillRemaining(
          hasScrollBody: false,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                const Spacer(),
                Text.rich(
                  context.l10n.errorScreenGenericDescription.toTextSpan(context),
                  textAlign: TextAlign.center,
                ),
                const Spacer(),
                ElevatedButton(
                  onPressed: () {
                    final settings = ModalRoute.of(context)?.settings;
                    if (settings != null) {
                      final args = getArgument(settings);
                      final loadEvent = HistoryDetailLoadTriggered(event: args.walletEvent);
                      context.read<HistoryDetailBloc>().add(loadEvent);
                    } else {
                      Navigator.pop(context);
                    }
                  },
                  child: Text.rich(context.l10n.generalRetry.toTextSpan(context)),
                ),
                const SizedBox(height: 16),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
