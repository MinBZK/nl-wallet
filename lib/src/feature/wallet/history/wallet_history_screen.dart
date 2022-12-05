import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_scroll_view.dart';
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
    return TimelineScrollView(attributes: state.attributes);
  }
}
