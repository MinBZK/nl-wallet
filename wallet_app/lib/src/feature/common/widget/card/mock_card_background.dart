import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:wallet_mock/mock.dart';

import '../../../../wallet_assets.dart';
import '../svg_or_image.dart';

class MockCardBackground extends StatelessWidget {
  final String attestationType;

  const MockCardBackground({required this.attestationType, super.key});

  @override
  Widget build(BuildContext context) {
    final bgAsset = _resolveBackgroundAsset(attestationType);
    if (bgAsset == null) return const DecoratedBox(decoration: BoxDecoration(color: Color(0xFFEEEFF7)));
    return SvgOrImage(asset: bgAsset, fit: BoxFit.cover, alignment: Alignment.topCenter);
  }

  String? _resolveBackgroundAsset(String attestationType) {
    switch (attestationType) {
      case MockAttestationTypes.pid:
        return WalletAssets.svg_rijks_card_bg_light;
      case MockAttestationTypes.address:
        return WalletAssets.svg_rijks_card_bg_dark;
      case MockAttestationTypes.bscDiploma:
      case MockAttestationTypes.mscDiploma:
        return WalletAssets.image_bg_diploma;
      case MockAttestationTypes.drivingLicense:
        return WalletAssets.image_bg_nl_driving_license;
      case MockAttestationTypes.healthInsurance:
        return WalletAssets.image_bg_health_insurance;
      case MockAttestationTypes.certificateOfConduct:
        return WalletAssets.image_bg_diploma;
    }
    Fimber.d('No mock background asset for: $attestationType');
    return null;
  }
}
