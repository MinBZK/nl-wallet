import 'package:flutter/material.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/mock.dart';

import '../../../../wallet_assets.dart';
import '../svg_or_image.dart';

class MockCardBackground extends StatelessWidget {
  final String docType;

  const MockCardBackground({required this.docType, super.key});

  @override
  Widget build(BuildContext context) {
    final bgAsset = _resolveBackgroundAsset(docType);
    if (bgAsset == null) return DecoratedBox(decoration: BoxDecoration(color: Color(0xFFEEEFF7)));
    return SvgOrImage(asset: bgAsset, fit: BoxFit.cover, alignment: Alignment.topCenter);
  }

  String? _resolveBackgroundAsset(String docType) {
    switch (docType) {
      case kPidDocType:
        return WalletAssets.svg_rijks_card_bg_light;
      case kAddressDocType:
        return WalletAssets.svg_rijks_card_bg_dark;
      case 'DIPLOMA_1':
        return WalletAssets.image_bg_diploma;
      case 'DIPLOMA_2':
        return WalletAssets.image_bg_diploma;
      case kDrivingLicenseDocType:
        return WalletAssets.image_bg_nl_driving_license;
      case 'HEALTH_INSURANCE':
        return WalletAssets.image_bg_health_insurance;
      case 'VOG':
        return WalletAssets.image_bg_diploma;
    }
    return null;
  }
}
