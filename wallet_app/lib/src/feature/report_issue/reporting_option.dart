import 'package:flutter/material.dart';

import '../../wallet_assets.dart';

enum ReportingOption {
  /// Organization
  unknownOrganization,
  impersonatingOrganization,
  suspiciousOrganization,
  overAskingOrganization,
  irrelevantAskingOrganization,

  /// Trust
  requestUntrusted,
  requestNotInitiated,

  /// Terms
  unreasonableTerms,

  /// Card
  incorrectCardData;

  const ReportingOption();

  Widget icon(Color? color) {
    switch (this) {
      /// Organization
      case ReportingOption.unknownOrganization:
        return _buildImageAsset(WalletAssets.icon_alert_unidentified_organization, color);
      case ReportingOption.impersonatingOrganization:
        return _buildImageAsset(WalletAssets.icon_alert_fake_id, color);
      case ReportingOption.suspiciousOrganization:
        return const Icon(Icons.apartment_outlined);
      case ReportingOption.overAskingOrganization:
        return _buildImageAsset(WalletAssets.icon_alert_data, color);
      case ReportingOption.irrelevantAskingOrganization:
        return _buildImageAsset(WalletAssets.icon_alert_data, color);

      /// Trust
      case ReportingOption.requestUntrusted:
        return const Icon(Icons.gpp_maybe_outlined);
      case ReportingOption.requestNotInitiated:
        return const Icon(Icons.gpp_maybe_outlined);

      /// Terms
      case ReportingOption.unreasonableTerms:
        return const Icon(Icons.handshake_outlined);

      /// Card
      case ReportingOption.incorrectCardData:
        return const Icon(Icons.credit_card_outlined);
    }
  }

  Image _buildImageAsset(String imageAsset, Color? color) {
    return Image.asset(
      imageAsset,
      color: color,
      colorBlendMode: BlendMode.srcIn,
    );
  }
}
