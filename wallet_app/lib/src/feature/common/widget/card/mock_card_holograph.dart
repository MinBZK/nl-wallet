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
    if (!Environment.mockRepositories) return const SizedBox.shrink();
    switch (attestationType) {
      case MockAttestationTypes.pid:
        return const CardHolograph(holograph: WalletAssets.svg_rijks_card_holo, brightness: Brightness.light);
      case MockAttestationTypes.address:
        return const CardHolograph(holograph: WalletAssets.svg_rijks_card_holo, brightness: Brightness.dark);
    }
    return const SizedBox.shrink();
  }
}
