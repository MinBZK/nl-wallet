// coverage:ignore-file
import 'package:flutter/material.dart';

import '../../common/widget/spacer/sliver_sized_box.dart';
import '../widget/digid_confirm_buttons.dart';
import '../widget/digid_sign_in_with_header.dart';
import '../widget/digid_sign_in_with_organization.dart';

class DigidConfirmAppPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onConfirmPressed;

  const DigidConfirmAppPage({
    required this.onConfirmPressed,
    required this.onDeclinePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: CustomScrollView(
          slivers: [
            const SliverToBoxAdapter(
              child: Align(
                alignment: Alignment.centerRight,
                child: Icon(Icons.close),
              ),
            ),
            const SliverToBoxAdapter(
              child: DigidSignInWithHeader(),
            ),
            const SliverSizedBox(height: 32),
            const SliverToBoxAdapter(
              child: DigidSignInWithOrganization(),
            ),
            const SliverSizedBox(height: 32),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: _buildBottomSection(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        DigidConfirmButtons(
          onAccept: onConfirmPressed,
          onDecline: onDeclinePressed,
        ),
      ],
    );
  }
}
