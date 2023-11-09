import 'dart:convert';

import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../../domain/model/navigation/navigation_request.dart';
import '../../../../domain/model/qr/edi_qr_code.dart';
import '../../../../feature/disclosure/argument/disclosure_screen_argument.dart';
import '../../../../feature/issuance/argument/issuance_screen_argument.dart';
import '../../../../feature/sign/argument/sign_screen_argument.dart';
import '../../../../navigation/wallet_routes.dart';
import '../qr_repository.dart';

const _kDefaultPrerequisites = [
  NavigationPrerequisite.walletUnlocked,
  NavigationPrerequisite.walletInitialized,
  NavigationPrerequisite.pidInitialized,
];

class MockQrRepository implements QrRepository {
  MockQrRepository();

  @override
  Future<NavigationRequest> processBarcode(Barcode barcode) => _processRawValue(barcode.rawValue!);

  Future<NavigationRequest> _processRawValue(rawValue) async {
    final json = jsonDecode(rawValue);
    final code = EdiQrCode.fromJson(json);
    switch (code.type) {
      case EdiQrType.issuance:
        return GenericNavigationRequest(
          WalletRoutes.issuanceRoute,
          argument: IssuanceScreenArgument(mockSessionId: code.id),
          navigatePrerequisites: _kDefaultPrerequisites,
        );
      case EdiQrType.disclosure:
        return GenericNavigationRequest(
          WalletRoutes.disclosureRoute,
          argument: DisclosureScreenArgument(mockSessionId: code.id),
          navigatePrerequisites: _kDefaultPrerequisites,
        );
      case EdiQrType.sign:
        return GenericNavigationRequest(
          WalletRoutes.signRoute,
          argument: SignScreenArgument(mockSessionId: code.id),
          navigatePrerequisites: _kDefaultPrerequisites,
        );
    }
  }
}
