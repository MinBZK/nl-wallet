import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/timeline_section.dart';
import '../../../util/timeline/timeline_section_list_factory.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_section_sliver.dart';
import '../../common/widget/text_icon_button.dart';
import 'bloc/wallet_history_bloc.dart';

class WalletHistoryScreen extends StatelessWidget {
  const WalletHistoryScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).walletHistoryScreenTitle),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<WalletHistoryBloc, WalletHistoryState>(
      builder: (context, state) {
        if (state is WalletHistoryInitial) return _buildLoading();
        if (state is WalletHistoryLoadInProgress) return _buildLoading();
        if (state is WalletHistoryLoadSuccess) return _buildTimeline(context, state);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildTimeline(BuildContext context, WalletHistoryLoadSuccess state) {
    final List<TimelineSection> sections = TimelineSectionListFactory.create(state.attributes);

    List<Widget> slivers = [
      ...sections.map((section) => TimelineSectionSliver(section: section)),
      _buildBackButton(context),
    ];

    return CustomScrollView(slivers: slivers);
  }

  Widget _buildBackButton(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      fillOverscroll: true,
      child: Align(
        alignment: Alignment.bottomCenter,
        child: SizedBox(
          height: 72,
          width: double.infinity,
          child: TextIconButton(
            onPressed: () => Navigator.pop(context),
            iconPosition: IconPosition.start,
            icon: Icons.arrow_back,
            child: Text(AppLocalizations.of(context).timelineScrollViewBackCta),
          ),
        ),
      ),
    );
  }
}
