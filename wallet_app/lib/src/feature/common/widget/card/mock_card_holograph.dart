import 'package:flutter/material.dart';
import 'package:wallet_mock/mock.dart';

import '../../../../../environment.dart';
import '../../../../wallet_assets.dart';
import 'card_holograph.dart';

/// A holograph that is only visible for the mock address and pid card
class MockCardHolograph extends StatelessWidget {
  final String attestationType;

  const MockCardHolograph({required this.attestationType, super.key});

  @override
  Widget build(BuildContext context) {
    final show = Environment.mockRepositories &&
        [MockConstants.pidDocType, MockConstants.addressDocType].contains(attestationType);
    if (!show) return const SizedBox.shrink();
    return CardHolograph(
      holograph: WalletAssets.svg_rijks_card_holo,
      // Taking shortcuts here to avoid adding extra info just for mock builds
      brightness: attestationType == MockConstants.pidDocType ? Brightness.light : Brightness.dark,
    );
  }
}
