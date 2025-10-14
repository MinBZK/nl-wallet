import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/data/repository/uri/core/core_uri_repository.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
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
      expect(result, NavigationRequest.disclosure(testUri));
    });

    test('Pid Issuance uri should result in a PidIssuanceNavigationRequest', () async {
      const testUri = 'https://pid_issuance.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.PidIssuance);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, NavigationRequest.pidIssuance(testUri));
    });

    test('Disclosure based issuance uri should result in a IssuanceNavigationRequest', () async {
      const testUri = 'https://disclosure_based_issuance.org';
      when(
        mockWalletCore.identifyUri(testUri),
      ).thenAnswer((realInvocation) async => IdentifyUriResult.DisclosureBasedIssuance);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, NavigationRequest.issuance(testUri));
    });

    test('Pid Renewal uri should result in a PidRenewalNavigationRequest', () async {
      const testUri = 'https://pid_renewal.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.PidRenewal);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, NavigationRequest.pidRenewal(testUri));
    });

    test('Pin Recovery uri should result in a PinRecoveryNavigationRequest', () async {
      const testUri = 'https://pin_recovery.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.PinRecovery);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, NavigationRequest.pinRecovery(testUri));
    });

    test('Transfer uri should result in a WalletTransferSourceNavigationRequest', () async {
      const testUri = 'https://transfer.org';
      when(mockWalletCore.identifyUri(testUri)).thenAnswer((realInvocation) async => IdentifyUriResult.Transfer);
      final result = await uriRepository.processUri(Uri.parse(testUri));
      expect(result, NavigationRequest.walletTransferSource(testUri));
    });
  });
}
