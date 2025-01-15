import 'dart:convert';

import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/environment.dart';
import 'package:wallet/src/data/repository/qr/core/core_qr_repository.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/qr/edi_qr_code.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late CoreQrRepository qrRepository;
  late MockTypedWalletCore mockWalletCore;

  setUp(() {
    mockWalletCore = MockTypedWalletCore();
    qrRepository = CoreQrRepository(mockWalletCore);
  });

  group('processBarcode', () {
    test('Disclosure QR code should result in a DisclosureNavigationRequest', () async {
      const testUri = 'https://disclosure.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.Disclosure);
      final result = await qrRepository.processBarcode(const Barcode(rawValue: testUri));
      expect(result, isA<DisclosureNavigationRequest>());
      expect(
        result.argument,
        const DisclosureScreenArgument(uri: testUri, isQrCode: true),
        reason: 'The original uri should be passed to the correct screen as an argument',
      );
    });

    test('Pid Issuance QR code should result in a PidIssuanceNavigationRequest', () async {
      const testUri = 'https://pid_issuance.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.PidIssuance);
      final result = await qrRepository.processBarcode(const Barcode(rawValue: testUri));
      expect(result, isA<PidIssuanceNavigationRequest>());
      expect(
        result.argument,
        testUri,
        reason: 'The original uri should be passed to the correct screen as an argument',
      );
    });

    test('Legacy Barcode is still supported (on mock builds)', () async {
      /// Create the json of a legacy EdiQrCode, as embedded in the mock QR codes
      const legacyQrCode = EdiQrCode(id: 'OPEN_BANK_ACCOUNT', type: EdiQrType.disclosure);
      final legacyQrAsJson = jsonEncode(legacyQrCode);

      /// Wrap it in a Barcode object, which is how the actual scanner would pass it through to the core
      final legacyQrAsBarcode = Barcode(rawValue: legacyQrAsJson);

      final result = qrRepository.legacyQrToDeeplink(
        legacyQrAsBarcode,
        forceConversion: true /* force since non-mock builds would skip this */,
      );
      expect(
        result,
        'walletdebuginteraction://deeplink#%7B%22id%22:%22OPEN_BANK_ACCOUNT%22,%22type%22:%22verify%22%7D',
        reason: 'The EdiQrCode should now be formatted as a URI',
      );
    });

    test('Legacy Barcode is ignored on non-mock builds', () async {
      expect(Environment.mockRepositories, isFalse, reason: 'test should be run with mock repositories disabled');

      /// Create the json of a legacy EdiQrCode, as embedded in the mock QR codes
      const legacyQrCode = EdiQrCode(id: 'OPEN_BANK_ACCOUNT', type: EdiQrType.disclosure);
      final legacyQrAsJson = jsonEncode(legacyQrCode);

      /// Wrap it in a Barcode object, which is how the actual scanner would pass it through to the core
      final legacyQrAsBarcode = Barcode(rawValue: legacyQrAsJson);

      final result = qrRepository.legacyQrToDeeplink(legacyQrAsBarcode);
      expect(
        result,
        isNull,
        reason:
            'When forceConversion is not set to true, no conversion of legacy urls should take place on non-mock builds.',
      );
    });
  });
}
