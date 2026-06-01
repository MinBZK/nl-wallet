import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../domain/model/result/application_error.dart';
import '../../navigation/wallet_routes.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../error/error_page.dart';
import 'argument/help_topic_screen_argument.dart';
import 'bloc/help_topic_bloc.dart';
import 'widget/topic_block_list.dart';

class HelpTopicScreen extends StatefulWidget {
  final HelpTopicScreenArgument argument;

  const HelpTopicScreen({required this.argument, super.key});

  @override
  State<HelpTopicScreen> createState() => _HelpTopicScreenState();
}

class _HelpTopicScreenState extends State<HelpTopicScreen> {
  // Skipped during keyboard traversal so the SelectionArea doesn't introduce
  // an invisible tab stop between the AppBar and the topic content; text
  // selection (and Ctrl+A/Ctrl+C once a selection exists) still works.
  final FocusNode _selectionFocusNode = FocusNode(skipTraversal: true);

  @override
  void dispose() {
    _selectionFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('helpTopicScreen'),
      appBar: WalletAppBar(
        title: BlocBuilder<HelpTopicBloc, HelpTopicState>(
          builder: (context, state) => TitleText(_titleFromState(state)),
        ),
      ),
      body: BlocBuilder<HelpTopicBloc, HelpTopicState>(
        builder: (context, state) => switch (state) {
          HelpTopicInitial() || HelpTopicLoadInProgress() => _buildScaffoldBody(_buildLoading()),
          HelpTopicLoadSuccess() => _buildScaffoldBody(_buildScrollableBody(_buildContent(context, state))),
          HelpTopicLoadFailure() => _buildError(context, state.error),
        },
      ),
    );
  }

  Widget _buildScaffoldBody(Widget child) {
    return SafeArea(
      child: Column(
        children: [
          Expanded(child: child),
          const BottomBackButton(),
        ],
      ),
    );
  }

  Widget _buildScrollableBody(Widget child) {
    return WalletScrollbar(
      child: SingleChildScrollView(
        padding: const EdgeInsets.only(bottom: 24),
        child: SelectionArea(
          focusNode: _selectionFocusNode,
          child: child,
        ),
      ),
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

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildContent(BuildContext context, HelpTopicLoadSuccess state) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: kDefaultTitlePadding,
          child: TitleText(state.title),
        ),
        TopicBlockList(
          blocks: state.blocks,
          onReferenceTap: (topicId) => _onReferenceTap(context, topicId),
        ),
      ],
    );
  }

  String _titleFromState(HelpTopicState state) => state is HelpTopicLoadSuccess ? state.title : '';

  Future<void> _onReferenceTap(BuildContext context, String targetTopicId) async {
    await Navigator.pushNamed(
      context,
      WalletRoutes.helpTopicRoute,
      arguments: HelpTopicScreenArgument(
        topicId: targetTopicId,
        visitedTopicIds: widget.argument.visitedTopicIds + [widget.argument.topicId],
      ),
    );
  }
}
