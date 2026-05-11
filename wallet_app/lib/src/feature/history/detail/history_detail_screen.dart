import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/result/application_error.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/utility/scroll_offset_provider.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../error/error_page.dart';
import 'argument/history_detail_screen_argument.dart';
import 'bloc/history_detail_bloc.dart';
import 'widget/page/history_detail_deletion_page.dart';
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
    return ScrollOffsetProvider(
      child: Scaffold(
        appBar: WalletAppBar(
          title: TitleText(_buildTitle(context)),
        ),
        key: const Key('historyDetailScreen'),
        body: SafeArea(
          child: BlocBuilder<HistoryDetailBloc, HistoryDetailState>(
            builder: (context, state) {
              return switch (state) {
                HistoryDetailInitial() => _buildLoading(context),
                HistoryDetailLoadInProgress() => _buildLoading(context),
                HistoryDetailLoadSuccess() => PrimaryScrollController(
                  controller: ScrollController(),
                  child: _buildSuccess(context, state),
                ),
                HistoryDetailLoadFailure() => _buildError(context, state.error),
              };
            },
          ),
        ),
      ),
    );
  }

  Widget _buildLoading(BuildContext context) {
    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(_buildTitle(context)),
          ),
        ),
        const SliverFillRemaining(
          hasScrollBody: false,
          child: CenteredLoadingIndicator(),
        ),
      ],
    );
  }

  String _buildTitle(BuildContext context) {
    final state = context.watch<HistoryDetailBloc>().state;
    final event = tryCast<HistoryDetailLoadSuccess>(state)?.event;
    if (event == null) return context.l10n.historyDetailScreenTitle;
    switch (event) {
      case DeletionEvent():
        return HistoryDetailDeletionPage.resolveTitle(context, event);
      case DisclosureEvent():
        switch (event.type) {
          case DisclosureType.regular:
            return HistoryDetailDisclosePage.resolveDisclosureTitle(context, event);
          case DisclosureType.login:
            return HistoryDetailLoginPage.resolveLoginTitle(context, event);
        }
      case IssuanceEvent():
        return HistoryDetailIssuePage.resolveTitle(context, event);
      case SignEvent():
        return context.l10n.historyDetailScreenTitle;
    }
  }

  Widget _buildSuccess(BuildContext context, HistoryDetailLoadSuccess state) {
    final WalletEvent event = state.event;
    final Widget page = switch (event) {
      DeletionEvent() => HistoryDetailDeletionPage(event: event),
      DisclosureEvent() => switch (event.type) {
        DisclosureType.regular => HistoryDetailDisclosePage(event: event),
        DisclosureType.login => HistoryDetailLoginPage(event: event),
      },
      IssuanceEvent() => HistoryDetailIssuePage(event: event),
      SignEvent() => HistoryDetailSignPage(event: event),
    };
    return Column(
      children: [
        Expanded(child: page),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildError(BuildContext context, ApplicationError error) {
    return ErrorPage.fromError(
      context,
      error,
      onPrimaryActionPressed: () => Navigator.pop(context),
      style: .close,
    );
  }
}
