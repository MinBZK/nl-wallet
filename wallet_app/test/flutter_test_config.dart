import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:visibility_detector/visibility_detector.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart' as domain;
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/pin/check_pin_result.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/start_sign_result/start_sign_result.dart';
import 'package:wallet/src/domain/model/tour/tour_video.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/domain/usecase/biometrics/biometric_authentication_result.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/helper/onboarding_helper.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet_core/core.dart' as core;

import 'src/mocks/core_mock_data.dart';
import 'src/mocks/wallet_mock_data.dart';
import 'src/mocks/wallet_mocks.dart';
import 'src/test_util/font_utils.dart';
import 'src/test_util/golden_diff_comparator.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await FontUtils.loadAppFonts();
  _provideDefaultCheckHasInternetMock();
  _setupMockitoDummies();
  _setupGoldenFileComparator();
  OnboardingHelper.initWithValue(8);
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

/// Configure mockito dummies for all test scenarios
void _setupMockitoDummies() {
  const stringValue = StringValue('');
  provideDummy<DataAttribute>(DataAttribute.untranslated(key: '', label: '', value: stringValue));
  provideDummy<AttributeValue>(stringValue);

  // Configuration
  provideDummy<FlutterAppConfiguration>(WalletMockData.flutterAppConfiguration);

  // Organization and policy dummies
  provideDummy<Organization>(WalletMockData.organization);
  provideDummy<Policy>(WalletMockData.policy);

  // Card status dummies
  provideDummy<CardStatus>(const CardStatusValid(validUntil: null));

  // Navigation request dummies
  provideDummy<NavigationRequest>(const GenericNavigationRequest('/mock_destination'));

  // Core wallet instruction result dummies
  provideDummy<core.WalletInstructionResult>(const core.WalletInstructionResult.ok());

  // Disclosure-related dummies
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
  provideDummy<DisclosureType>(DisclosureType.login);
  provideDummy<DisclosureSessionType>(DisclosureSessionType.crossDevice);
  provideDummy<core.RevocationCodeResult>(const core.RevocationCodeResult_Ok(revocationCode: '123456'));
  provideDummy<core.AcceptDisclosureResult>(const core.AcceptDisclosureResult_Ok());
  provideDummy<core.StartDisclosureResult>(
    const core.StartDisclosureResult.requestAttributesMissing(
      relyingParty: core.Organization(legalName: [], displayName: [], description: [], category: []),
      missingAttributes: [],
      requestOriginBaseUrl: '',
      sharedDataWithRelyingPartyBefore: false,
      sessionType: core.DisclosureSessionType.CrossDevice,
      requestPurpose: [],
    ),
  );
  provideDummy<core.DisclosureBasedIssuanceResult>(const core.DisclosureBasedIssuanceResult_Ok([]));

  // Issuance-related dummies
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
  provideDummy<core.PidIssuanceResult>(const core.PidIssuanceResult.ok(transferAvailable: true));

  // Signing-related dummies
  provideDummy<StartSignResult>(
    StartSignReadyToSign(
      document: WalletMockData.document,
      policy: WalletMockData.policy,
      relyingParty: WalletMockData.organization,
      trustProvider: WalletMockData.organization,
      requestedCards: [],
    ),
  );

  // Wallet event dummies
  provideDummy<WalletEvent>(
    WalletEvent.issuance(
      dateTime: DateTime.now(),
      status: EventStatus.success,
      card: WalletMockData.card,
      eventType: IssuanceEventType.cardIssued,
    ),
  );

  // Wallet state dummies
  provideDummy<core.WalletState>(const core.WalletState_Ready());
  provideDummy<WalletState>(const WalletStateReady());

  // Pin-related dummies
  provideDummy<CheckPinResult>(CheckPinResultBlocked());

  // Error dummies
  provideDummy<CoreError>(const CoreGenericError('dummy', data: {}));

  // Result dummies - basic types
  provideDummy<Result<dynamic>>(const Result.success(null));
  provideDummy<Result<void>>(const Result.success(null));
  provideDummy<Result<String>>(const Result.success(''));
  provideDummy<Result<String?>>(const Result.success(null));
  provideDummy<Result<bool>>(const Result.success(true));

  // Result dummies - collections
  provideDummy<Result<List<WalletEvent>>>(const Result.success([]));
  provideDummy<Result<List<Attribute>>>(const Result.success([]));
  provideDummy<Result<List<WalletCard>>>(const Result.success([]));
  provideDummy<Result<List<TourVideo>>>(const Result.success([]));

  // Result dummies - specific objects
  provideDummy<Result<NavigationRequest>>(const Result.success(GenericNavigationRequest('/mock_destination')));
  provideDummy<Result<StartDisclosureResult>>(
    Result.success(
      StartDisclosureReadyToDisclose(
        relyingParty: WalletMockData.organization,
        originUrl: 'http://origin.org',
        requestPurpose: 'requestPurpose'.untranslated,
        sessionType: DisclosureSessionType.crossDevice,
        type: DisclosureType.login,
        policy: WalletMockData.policy,
        sharedDataWithOrganizationBefore: false,
        cardRequests: [],
      ),
    ),
  );
  provideDummy<Result<domain.StartIssuanceResult>>(
    Result.success(
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
    ),
  );
  provideDummy<Result<StartSignResult>>(
    Result.success(
      StartSignReadyToSign(
        document: WalletMockData.document,
        policy: WalletMockData.policy,
        relyingParty: WalletMockData.organization,
        trustProvider: WalletMockData.organization,
        requestedCards: [],
      ),
    ),
  );
  provideDummy<Result<WalletCard>>(Result.success(WalletMockData.card));
  provideDummy<Result<BiometricAuthenticationResult>>(const Result.success(BiometricAuthenticationResult.success));

  // Result dummies - Notifications
  provideDummy<AppNotification>(
    AppNotification(
      id: 0,
      type: NotificationType.cardRevoked(card: WalletMockData.card),
      displayTargets: [const .dashboard()],
    ),
  );
  provideDummy<NotificationType>(.cardExpired(card: WalletMockData.card));
  provideDummy<core.NotificationType>(const core.NotificationType.cardExpired(card: CoreMockData.attestation));
}

/// Overrides the default [LocalFileComparator] with our [GoldenDiffComparator] that has
/// a configurable tolerance (defaults to 0.5%) when comparing goldens.
void _setupGoldenFileComparator() {
  final testFilePath = (goldenFileComparator as LocalFileComparator).basedir;
  goldenFileComparator = GoldenDiffComparator(testFilePath.toString());
}
