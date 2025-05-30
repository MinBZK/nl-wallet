import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/data/repository/uri/core/core_uri_repository.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockTypedWalletCore mockWalletCore;
  late CoreUriRepository uriRepository;

  setUp(() {
    mockWalletCore = MockTypedWalletCore();
    uriRepository = CoreUriRepository(mockWalletCore);
  });

  group('processUri', () {
    test('Disclosure uri should result in a DisclosureNavigationRequest', () async {
      const testUri = 'https://disclosure.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.Disclosure);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, isA<DisclosureNavigationRequest>());
      expect(
        result.argument,
        const DisclosureScreenArgument(uri: testUri, isQrCode: false),
        reason: 'The original uri should be passed to the correct screen as an argument',
      );
    });

    test('Pid Issuance uri should result in a PidIssuanceNavigationRequest', () async {
      const testUri = 'https://pid_issuance.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.PidIssuance);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, isA<PidIssuanceNavigationRequest>());
      expect(
        result.argument,
        testUri,
        reason: 'The original uri should be passed to the correct screen as an argument',
      );
    });

    test('Disclosure based issuance uri should result in a IssuanceNavigationRequest', () async {
      const testUri = 'https://disclosure_based_issuance.org';
      when(mockWalletCore.identifyUri(testUri))
          .thenAnswer((realInvocation) async => IdentifyUriResult.DisclosureBasedIssuance);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, isA<IssuanceNavigationRequest>());
      expect(
        result.argument,
        const IssuanceScreenArgument(uri: testUri, isRefreshFlow: false, isQrCode: false),
        reason: 'The original uri should be passed to the correct screen as an argument',
      );
    });
  });
}
