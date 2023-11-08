import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/qr/core/core_qr_repository.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';

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
      expect(result.argument, const DisclosureScreenArgument(uri: testUri),
          reason: 'The original uri should be passed to the correct screen as an argument');
    });

    test('Pid Issuance QR code should result in a PidIssuanceNavigationRequest', () async {
      const testUri = 'https://pid_issuance.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.PidIssuance);
      final result = await qrRepository.processBarcode(const Barcode(rawValue: testUri));
      expect(result, isA<PidIssuanceNavigationRequest>());
      expect(result.argument, testUri,
          reason: 'The original uri should be passed to the correct screen as an argument');
    });
  });
}
