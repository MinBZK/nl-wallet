import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/disclosure/core/core_disclosure_repository.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart' as ui;
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CoreDisclosureRepository repository;
  late MockTypedWalletCore mockTypedWalletCore;

  setUp(() {
    mockTypedWalletCore = MockTypedWalletCore();
    repository = CoreDisclosureRepository(
      mockTypedWalletCore,
      MockMapper(),
      MockMapper(),
      MockMapper(),
      MockMapper(),
      MockMapper(),
      MockMapper(),
      MockMapper(),
    );
  });

  test('Call to cancelDisclosure is forwarded to wallet core', () async {
    await repository.cancelDisclosure();
    verify(mockTypedWalletCore.cancelDisclosure()).called(1);
  });

  test('Call to acceptDisclosure is forwarded to wallet core with correct argument', () async {
    await repository.acceptDisclosure('123123', [0]);
    verify(mockTypedWalletCore.acceptDisclosure('123123', [0])).called(1);
  });

  test('Call to startDisclosure is forwarded to wallet core with correct argument', () async {
    await repository.startDisclosure('uri', isQrCode: true);
    verify(mockTypedWalletCore.startDisclosure('uri', isQrCode: true)).called(1);
  });

  test('StartDisclosureResultRequest is mapped successfully to StartDisclosureReadyToDisclose', () async {
    when(mockTypedWalletCore.startDisclosure(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
      return const StartDisclosureResult.request(
        relyingParty: Organization(legalName: [], displayName: [], description: [], category: []),
        requestOriginBaseUrl: '',
        sharedDataWithRelyingPartyBefore: false,
        sessionType: DisclosureSessionType.CrossDevice,
        requestPurpose: [],
        policy: RequestPolicy(
          dataSharedWithThirdParties: false,
          dataDeletionPossible: true,
          policyUrl: 'https://example.org',
        ),
        requestType: DisclosureType.Login,
        disclosureOptions: [],
      );
    });
    final result = await repository.startDisclosure('uri', isQrCode: true);
    expect(result, isA<ui.StartDisclosureReadyToDisclose>());
  });
}
