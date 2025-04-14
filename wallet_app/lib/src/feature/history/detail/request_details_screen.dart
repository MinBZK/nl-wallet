import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/object_extension.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/wallet_scrollbar.dart';
import 'widget/history_detail_common_builders.dart';

class RequestDetailsScreen extends StatelessWidget {
  final DisclosureEvent event;

  const RequestDetailsScreen({required this.event, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.requestDetailScreenTitle,
                      scrollController: PrimaryScrollController.maybeOf(context),
                    ),
                    HistoryDetailCommonBuilders.buildPurposeSliver(context, event),
                    HistoryDetailCommonBuilders.buildRequestedAttributesSliver(context, event)
                        .takeIf((_) => event.sharedAttributes.isNotEmpty),
                    HistoryDetailCommonBuilders.buildPolicySliver(context, event.relyingParty, event.policy),
                    const SliverSizedBox(height: 24),
                  ].nonNulls.toList(),
                ),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  static void show(BuildContext context, DisclosureEvent event) {
    Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => RequestDetailsScreen(event: event),
      ),
    );
  }
}
