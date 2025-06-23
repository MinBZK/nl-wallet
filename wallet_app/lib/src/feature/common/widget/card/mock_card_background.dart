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

  String? _resolveBackgroundAsset(String docType) {
    switch (docType) {
      case MockConstants.pidDocType:
        return WalletAssets.svg_rijks_card_bg_light;
      case MockConstants.addressDocType:
        return WalletAssets.svg_rijks_card_bg_dark;
      case 'DIPLOMA_1':
        return WalletAssets.image_bg_diploma;
      case 'DIPLOMA_2':
        return WalletAssets.image_bg_diploma;
      case MockConstants.drivingLicenseDocType:
        return WalletAssets.image_bg_nl_driving_license;
      case 'HEALTH_INSURANCE':
        return WalletAssets.image_bg_health_insurance;
      case 'VOG':
        return WalletAssets.image_bg_diploma;
    }
    return null;
  }
}
