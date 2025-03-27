import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/disclosure/core/core_disclosure_repository.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart' as ui;
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CoreDisclosureRepository repository;
  late MockTypedWalletCore mockTypedWalletCore;

  setUp(() {
    provideDummy<AcceptDisclosureResult>(AcceptDisclosureResult_Ok());
    provideDummy<ui.DisclosureSessionType>(ui.DisclosureSessionType.crossDevice);
    provideDummy<StartDisclosureResult>(
      StartDisclosureResult.requestAttributesMissing(
        relyingParty: Organization(legalName: [], displayName: [], description: [], category: []),
        missingAttributes: [],
        requestOriginBaseUrl: '',
        sharedDataWithRelyingPartyBefore: false,
        sessionType: DisclosureSessionType.CrossDevice,
        requestPurpose: [],
      ),
    );
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

  test('Call to hasActiveDisclosureSession is forwarded to wallet core', () async {
    await repository.hasActiveDisclosureSession();
    verify(mockTypedWalletCore.hasActiveDisclosureSession()).called(1);
  });

  test('Call to acceptDisclosure is forwarded to wallet core with correct argument', () async {
    await repository.acceptDisclosure('123123');
    verify(mockTypedWalletCore.acceptDisclosure('123123')).called(1);
  });

  test('Call to startDisclosure is forwarded to wallet core with correct argument', () async {
    await repository.startDisclosure('uri', isQrCode: true);
    verify(mockTypedWalletCore.startDisclosure('uri', isQrCode: true)).called(1);
  });
}
