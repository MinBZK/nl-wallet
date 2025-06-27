import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:visibility_detector/visibility_detector.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart' as domain;
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/start_sign_result/start_sign_result.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet_core/core.dart' as core;

import 'src/mocks/wallet_mock_data.dart';
import 'src/mocks/wallet_mocks.dart';
import 'src/test_util/font_utils.dart';
import 'src/test_util/golden_diff_comparator.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await FontUtils.loadAppFonts();
  _provideDefaultCheckHasInternetMock();
  _setupMockitoDummies();
  _setupGoldenFileComparator();
  VisibilityDetectorController.instance.updateInterval = Duration.zero;
  return testMain();
}

/// Mapping [CoreError]s to [ApplicationError]s relies on the static [CoreErrorExtension.networkRepository],
/// provide a default implementation for all tests.
void _provideDefaultCheckHasInternetMock() {
  final mockNetworkRepository = MockNetworkRepository();
  when(mockNetworkRepository.hasInternet()).thenAnswer((realInvocation) async => true);
  CoreErrorExtension.networkRepository = mockNetworkRepository;
}

/// Configure some basic mockito dummies
void _setupMockitoDummies() {
  provideDummy<DataAttribute>(
    DataAttribute.untranslated(key: '', label: '', value: const StringValue(''), sourceCardDocType: ''),
  );
  provideDummy<Organization>(WalletMockData.organization);
  provideDummy<AttributeValue>(const StringValue(''));
  provideDummy<NavigationRequest>(const GenericNavigationRequest('/mock_destination'));
  provideDummy<core.WalletInstructionResult>(const core.WalletInstructionResult.ok());
  provideDummy<StartDisclosureResult>(
    StartDisclosureReadyToDisclose(
      relyingParty: WalletMockData.organization,
      originUrl: 'http://origin.org',
      requestPurpose: 'requestPurpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      type: DisclosureType.regular,
      policy: WalletMockData.policy,
      sharedDataWithOrganizationBefore: false,
      cardRequests: [],
    ),
  );
  provideDummy<CoreError>(const CoreGenericError('dummy', data: {}));
  provideDummy<domain.StartIssuanceResult>(
    domain.StartIssuanceReadyToDisclose(
      relyingParty: WalletMockData.organization,
      policy: WalletMockData.policy,
      sessionType: DisclosureSessionType.sameDevice,
      cardRequests: [],
      originUrl: '',
      requestPurpose: {},
      type: DisclosureType.regular,
      sharedDataWithOrganizationBefore: false,
    ),
  );
  provideDummy<StartSignResult>(
    StartSignReadyToSign(
      document: WalletMockData.document,
      policy: WalletMockData.policy,
      relyingParty: WalletMockData.organization,
      trustProvider: WalletMockData.organization,
      requestedCards: [],
    ),
  );
  // Provide some basic [Result] dummies, anything more specific should be defined in the test itself.
  provideDummy<Result<dynamic>>(const Result.success(null));
  provideDummy<Result<void>>(const Result.success(null));
  provideDummy<Result<String>>(const Result.success(''));
  provideDummy<Result<String?>>(const Result.success(null));
}

/// Overrides the default [LocalFileComparator] with our [GoldenDiffComparator] that has
/// a configurable tolerance (defaults to 0.5%) when comparing goldens.
void _setupGoldenFileComparator() {
  final testFilePath = (goldenFileComparator as LocalFileComparator).basedir;
  goldenFileComparator = GoldenDiffComparator(testFilePath.toString());
}
