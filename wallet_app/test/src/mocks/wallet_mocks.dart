import 'package:bluetooth/bluetooth.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:get_it/get_it.dart';
import 'package:internet_connection_checker/internet_connection_checker.dart';
import 'package:local_auth/local_auth.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:video_player/video_player.dart';
import 'package:wallet/src/data/repository/biometric/biometric_repository.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/close_proximity/close_proximity_repository.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/data/repository/event/wallet_event_repository.dart';
import 'package:wallet/src/data/repository/issuance/issuance_repository.dart';
import 'package:wallet/src/data/repository/language/language_repository.dart';
import 'package:wallet/src/data/repository/network/network_repository.dart';
import 'package:wallet/src/data/repository/notification/notification_repository.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/data/repository/pin/pin_repository.dart';
import 'package:wallet/src/data/repository/revocation/revocation_code_repository.dart';
import 'package:wallet/src/data/repository/tour/tour_repository.dart';
import 'package:wallet/src/data/repository/transfer/transfer_repository.dart';
import 'package:wallet/src/data/repository/version/version_state_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/data/service/announcement_service.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/data/service/auto_lock_service.dart';
import 'package:wallet/src/data/service/event/app_event_coordinator.dart';
import 'package:wallet/src/data/service/local_notification_service.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/data/service/semantics_event_service.dart';
import 'package:wallet/src/data/store/active_locale_provider.dart';
import 'package:wallet/src/data/store/notification_settings_store.dart';
import 'package:wallet/src/data/store/revocation_code_store.dart';
import 'package:wallet/src/domain/app_event/app_event_listener.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/request_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/set_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/card/delete_wallet_card_usecase.dart';
import 'package:wallet/src/domain/usecase/card/get_pid_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/card/get_wallet_card_usecase.dart';
import 'package:wallet/src/domain/usecase/card/get_wallet_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_detail_usecase.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_usecase.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/close_proximity/observe_close_proximity_connection_usecase.dart';
import 'package:wallet/src/domain/usecase/close_proximity/start_close_proximity_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/accept_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/start_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/event/get_most_recent_wallet_event_usecase.dart';
import 'package:wallet/src/domain/usecase/event/get_wallet_events_for_card_usecase.dart';
import 'package:wallet/src/domain/usecase/event/get_wallet_events_usecase.dart';
import 'package:wallet/src/domain/usecase/event/observe_recent_wallet_events_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/accept_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/cancel_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/start_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/maintenance/observe_maintenance_state_usecase.dart';
import 'package:wallet/src/domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import 'package:wallet/src/domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/observe_dashboard_notifications_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/observe_os_notifications_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/observe_push_notifications_setting_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/set_direct_os_notification_callback_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/set_push_notifications_setting_usecase.dart';
import 'package:wallet/src/domain/usecase/permission/check_permission_usecase.dart';
import 'package:wallet/src/domain/usecase/permission/request_permission_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/accept_offered_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/check_is_pid.dart';
import 'package:wallet/src/domain/usecase/pid/continue_pid_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/get_pid_renewal_url_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/cancel_pin_recovery_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/change_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/check_is_valid_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/complete_pin_recovery_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/continue_pin_recovery_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/create_pin_recovery_url_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/disclose_for_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/qr/decode_qr_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/get_registration_revocation_code_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/get_revocation_code_saved_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/get_revocation_code_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/set_revocation_code_saved_usecase.dart';
import 'package:wallet/src/domain/usecase/sign/accept_sign_agreement_usecase.dart';
import 'package:wallet/src/domain/usecase/sign/reject_sign_agreement_usecase.dart';
import 'package:wallet/src/domain/usecase/sign/start_sign_usecase.dart';
import 'package:wallet/src/domain/usecase/tour/fetch_tour_videos_usecase.dart';
import 'package:wallet/src/domain/usecase/tour/observe_show_tour_banner_usecase.dart';
import 'package:wallet/src/domain/usecase/tour/tour_overview_viewed_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/confirm_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/init_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/observe_transfer_session_state_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/pair_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/receive_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/skip_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/start_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/update/observe_version_state_usecase.dart';
import 'package:wallet/src/domain/usecase/uri/decode_uri_usecase.dart';
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/create_wallet_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/get_wallet_state_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_registered_and_unlocked_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/lock_wallet_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/move_to_ready_state_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/reset_wallet_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet_core/core.dart';
import 'package:workmanager/workmanager.dart';

import 'wallet_mocks.mocks.dart';

export 'wallet_mocks.mocks.dart';

/// Definition of mocks used by our tests. When specifying new mocks makes sure to run:
///
/// dart run build_runner build --delete-conflicting-outputs
/// dart format . --line-length 120
///
/// to generate and format the new mocks.

/// Mock framework
@GenerateNiceMocks([MockSpec<BuildContext>()])
@GenerateNiceMocks([MockSpec<GlobalKey<NavigatorState>>(as: Symbol('MockNavigatorKey'))])
@GenerateNiceMocks([MockSpec<InternetConnectionChecker>()])
@GenerateNiceMocks([MockSpec<NavigatorState>()])
@GenerateNiceMocks([MockSpec<VideoPlayerController>()])
/// Mock mappers
@GenerateNiceMocks([MockSpec<ContextMapper>()])
@GenerateNiceMocks([MockSpec<Mapper>()])
/// Mock repositories
@GenerateNiceMocks([MockSpec<BiometricRepository>()])
@GenerateNiceMocks([MockSpec<ConfigurationRepository>()])
@GenerateNiceMocks([MockSpec<DisclosureRepository>()])
@GenerateNiceMocks([MockSpec<IssuanceRepository>()])
@GenerateNiceMocks([MockSpec<LanguageRepository>()])
@GenerateNiceMocks([MockSpec<NotificationRepository>()])
@GenerateNiceMocks([MockSpec<PidRepository>()])
@GenerateNiceMocks([MockSpec<PinRepository>()])
@GenerateNiceMocks([MockSpec<RevocationRepository>()])
@GenerateNiceMocks([MockSpec<TourRepository>()])
@GenerateNiceMocks([MockSpec<TransferRepository>()])
@GenerateNiceMocks([MockSpec<VersionStateRepository>()])
@GenerateNiceMocks([MockSpec<WalletCardRepository>()])
@GenerateNiceMocks([MockSpec<WalletEventRepository>()])
@GenerateNiceMocks([MockSpec<WalletRepository>()])
/// Mock services
@GenerateNiceMocks([MockSpec<ActiveLocaleProvider>()])
@GenerateNiceMocks([MockSpec<AnnouncementService>()])
@GenerateNiceMocks([MockSpec<AppEventCoordinator>()])
@GenerateNiceMocks([MockSpec<AppEventListener>()])
@GenerateNiceMocks([MockSpec<AutoLockService>()])
@GenerateNiceMocks([MockSpec<BiometricUnlockManager>()])
@GenerateNiceMocks([MockSpec<Bluetooth>()])
@GenerateNiceMocks([MockSpec<CloseProximityRepository>()])
@GenerateNiceMocks([MockSpec<FlutterLocalNotificationsPlugin>()])
@GenerateNiceMocks([MockSpec<LocalAuthentication>()])
@GenerateNiceMocks([MockSpec<LocalNotificationService>()])
@GenerateNiceMocks([MockSpec<NavigationService>()])
@GenerateNiceMocks([MockSpec<NotificationSettingsStore>()])
@GenerateNiceMocks([MockSpec<RevocationCodeStore>()])
@GenerateNiceMocks([MockSpec<SemanticsEventService>()])
@GenerateNiceMocks([MockSpec<TypedWalletCore>()])
@GenerateNiceMocks([MockSpec<WalletCoreApi>()])
@GenerateNiceMocks([MockSpec<Workmanager>()])
/// Mock use cases
@GenerateNiceMocks([MockSpec<AcceptDisclosureUseCase>()])
@GenerateNiceMocks([MockSpec<AcceptIssuanceUseCase>()])
@GenerateNiceMocks([MockSpec<AcceptOfferedPidUseCase>()])
@GenerateNiceMocks([MockSpec<AcceptSignAgreementUseCase>()])
@GenerateNiceMocks([MockSpec<CancelDisclosureUseCase>()])
@GenerateNiceMocks([MockSpec<CancelIssuanceUseCase>()])
@GenerateNiceMocks([MockSpec<CancelPidIssuanceUseCase>()])
@GenerateNiceMocks([MockSpec<CancelPinRecoveryUseCase>()])
@GenerateNiceMocks([MockSpec<CancelWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<ChangePinUseCase>()])
@GenerateNiceMocks([MockSpec<CheckIsPidUseCase>()])
@GenerateNiceMocks([MockSpec<CheckIsValidPinUseCase>()])
@GenerateNiceMocks([MockSpec<CheckNavigationPrerequisitesUseCase>()])
@GenerateNiceMocks([MockSpec<CheckPermissionUseCase>()])
@GenerateNiceMocks([MockSpec<CheckPinUseCase>()])
@GenerateNiceMocks([MockSpec<CompletePinRecoveryUseCase>()])
@GenerateNiceMocks([MockSpec<ConfirmWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<ContinuePidIssuanceUseCase>()])
@GenerateNiceMocks([MockSpec<ContinuePinRecoveryUseCase>()])
@GenerateNiceMocks([MockSpec<CreatePinRecoveryRedirectUriUseCase>()])
@GenerateNiceMocks([MockSpec<CreateWalletUseCase>()])
@GenerateNiceMocks([MockSpec<DecodeQrUseCase>()])
@GenerateNiceMocks([MockSpec<DecodeUriUseCase>()])
@GenerateNiceMocks([MockSpec<DeleteWalletCardUseCase>()])
@GenerateNiceMocks([MockSpec<DiscloseForIssuanceUseCase>()])
@GenerateNiceMocks([MockSpec<FetchTourVideosUseCase>()])
@GenerateNiceMocks([MockSpec<GetAvailableBiometricsUseCase>()])
@GenerateNiceMocks([MockSpec<GetMostRecentWalletEventUseCase>()])
@GenerateNiceMocks([MockSpec<GetPidCardsUseCase>()])
@GenerateNiceMocks([MockSpec<GetPidIssuanceUrlUseCase>()])
@GenerateNiceMocks([MockSpec<GetPidRenewalUrlUseCase>()])
@GenerateNiceMocks([MockSpec<GetRegistrationRevocationCodeUseCase>()])
@GenerateNiceMocks([MockSpec<GetRevocationCodeSavedUseCase>()])
@GenerateNiceMocks([MockSpec<GetRevocationCodeUseCase>()])
@GenerateNiceMocks([MockSpec<GetSupportedBiometricsUseCase>()])
@GenerateNiceMocks([MockSpec<GetVersionStringUseCase>()])
@GenerateNiceMocks([MockSpec<GetWalletCardUseCase>()])
@GenerateNiceMocks([MockSpec<GetWalletCardsUseCase>()])
@GenerateNiceMocks([MockSpec<GetWalletEventsForCardUseCase>()])
@GenerateNiceMocks([MockSpec<GetWalletEventsUseCase>()])
@GenerateNiceMocks([MockSpec<GetWalletStateUseCase>()])
@GenerateNiceMocks([MockSpec<InitWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<IsBiometricLoginEnabledUseCase>()])
@GenerateNiceMocks([MockSpec<IsWalletInitializedUseCase>()])
@GenerateNiceMocks([MockSpec<IsWalletInitializedWithPidUseCase>()])
@GenerateNiceMocks([MockSpec<IsWalletRegisteredAndUnlockedUseCase>()])
@GenerateNiceMocks([MockSpec<LockWalletUseCase>()])
@GenerateNiceMocks([MockSpec<MoveToReadyStateUseCase>()])
@GenerateNiceMocks([MockSpec<NetworkRepository>()])
@GenerateNiceMocks([MockSpec<ObserveCloseProximityConnectionUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveDashboardNotificationsUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveMaintenanceStateUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveOsNotificationsUseCase>()])
@GenerateNiceMocks([MockSpec<ObservePushNotificationsSettingUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveRecentWalletEventsUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveShowTourBannerUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveTransferSessionStateUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveVersionStateUsecase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletCardDetailUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletCardUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletCardsUseCase>()])
@GenerateNiceMocks([MockSpec<ObserveWalletLockedUseCase>()])
@GenerateNiceMocks([MockSpec<PairWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<PerformPreNavigationActionsUseCase>()])
@GenerateNiceMocks([MockSpec<ReceiveWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<RejectSignAgreementUseCase>()])
@GenerateNiceMocks([MockSpec<RequestBiometricsUseCase>()])
@GenerateNiceMocks([MockSpec<RequestPermissionUseCase>()])
@GenerateNiceMocks([MockSpec<ResetWalletUseCase>()])
@GenerateNiceMocks([MockSpec<SetBiometricsUseCase>()])
@GenerateNiceMocks([MockSpec<SetDirectOsNotificationCallbackUsecase>()])
@GenerateNiceMocks([MockSpec<SetPushNotificationsSettingUseCase>()])
@GenerateNiceMocks([MockSpec<SetRevocationCodeSavedUseCase>()])
@GenerateNiceMocks([MockSpec<SetupMockedWalletUseCase>()])
@GenerateNiceMocks([MockSpec<SkipWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<StartCloseProximityDisclosureUseCase>()])
@GenerateNiceMocks([MockSpec<StartDisclosureUseCase>()])
@GenerateNiceMocks([MockSpec<StartIssuanceUseCase>()])
@GenerateNiceMocks([MockSpec<StartSignUseCase>()])
@GenerateNiceMocks([MockSpec<StartWalletTransferUseCase>()])
@GenerateNiceMocks([MockSpec<TourOverviewViewedUseCase>()])
@GenerateNiceMocks([MockSpec<UnlockWalletWithPinUseCase>()])
/// Constants
const kMockPidIssuanceUrl = 'https://example.org';

/// Class that provides the generated mocks with very
/// basic, or no stubbing. Stubs can be overwritten or the mocks
/// can always be instantiated directly. The main intention here is
/// to allow us to instantiate classes under tests in a simple way,
/// i.e. `xxRepository(Mocks.create(), Mocks.create(), Mocks.create())`
/// When you need more control over what a mock returns you should
/// probably instantiate the mock directly in your test class.
class Mocks {
  Mocks._();

  static final sl = GetIt.asNewInstance();
  static var isInitialized = false;

  static void initialize() {
    // Core
    sl.registerFactory<WalletCoreApi>(MockWalletCoreApi.new);

    // Framework
    sl.registerFactory<BuildContext>(MockBuildContext.new);
    sl.registerFactory<GlobalKey<NavigatorState>>(MockNavigatorKey.new);
    sl.registerFactory<InternetConnectionChecker>(MockInternetConnectionChecker.new);
    sl.registerFactory<NavigatorState>(MockNavigatorState.new);
    sl.registerFactory<VideoPlayerController>(MockVideoPlayerController.new);

    // Mappers
    sl.registerFactory<ContextMapper>(MockContextMapper.new);
    sl.registerFactory<Mapper>(MockMapper.new);

    // Repositories
    sl.registerFactory<BiometricRepository>(MockBiometricRepository.new);
    sl.registerFactory<CloseProximityRepository>(MockCloseProximityRepository.new);
    sl.registerFactory<ConfigurationRepository>(_createConfigurationRepositoryMock);
    sl.registerFactory<DisclosureRepository>(MockDisclosureRepository.new);
    sl.registerFactory<IssuanceRepository>(MockIssuanceRepository.new);
    sl.registerFactory<LanguageRepository>(MockLanguageRepository.new);
    sl.registerFactory<NetworkRepository>(_createNetworkRepositoryMock);
    sl.registerFactory<NotificationRepository>(MockNotificationRepository.new);
    sl.registerFactory<PidRepository>(_createPidRepositoryMock);
    sl.registerFactory<PinRepository>(MockPinRepository.new);
    sl.registerFactory<RevocationRepository>(MockRevocationRepository.new);
    sl.registerFactory<TourRepository>(MockTourRepository.new);
    sl.registerFactory<TransferRepository>(MockTransferRepository.new);
    sl.registerFactory<VersionStateRepository>(MockVersionStateRepository.new);
    sl.registerFactory<WalletCardRepository>(MockWalletCardRepository.new);
    sl.registerFactory<WalletEventRepository>(MockWalletEventRepository.new);
    sl.registerFactory<WalletRepository>(MockWalletRepository.new);

    // Services
    sl.registerFactory<ActiveLocaleProvider>(MockActiveLocaleProvider.new);
    sl.registerFactory<AnnouncementService>(MockAnnouncementService.new);
    sl.registerFactory<AppEventCoordinator>(MockAppEventCoordinator.new);
    sl.registerFactory<AppEventListener>(MockAppEventListener.new);
    sl.registerFactory<AppLifecycleService>(AppLifecycleService.new);
    sl.registerFactory<AutoLockService>(MockAutoLockService.new);
    sl.registerFactory<BiometricUnlockManager>(MockBiometricUnlockManager.new);
    sl.registerFactory<Bluetooth>(MockBluetooth.new);
    sl.registerFactory<FlutterLocalNotificationsPlugin>(MockFlutterLocalNotificationsPlugin.new);
    sl.registerFactory<LocalAuthentication>(MockLocalAuthentication.new);
    sl.registerFactory<LocalNotificationService>(MockLocalNotificationService.new);
    sl.registerFactory<NavigationService>(MockNavigationService.new);
    sl.registerFactory<NotificationSettingsStore>(MockNotificationSettingsStore.new);
    sl.registerFactory<RevocationCodeStore>(MockRevocationCodeStore.new);
    sl.registerFactory<SemanticsEventService>(MockSemanticsEventService.new);
    sl.registerFactory<TypedWalletCore>(_createTypedWalletCoreMock);
    sl.registerFactory<Workmanager>(MockWorkmanager.new);

    // Use cases
    sl.registerFactory<AcceptDisclosureUseCase>(MockAcceptDisclosureUseCase.new);
    sl.registerFactory<AcceptIssuanceUseCase>(MockAcceptIssuanceUseCase.new);
    sl.registerFactory<AcceptOfferedPidUseCase>(MockAcceptOfferedPidUseCase.new);
    sl.registerFactory<AcceptSignAgreementUseCase>(MockAcceptSignAgreementUseCase.new);
    sl.registerFactory<CancelDisclosureUseCase>(MockCancelDisclosureUseCase.new);
    sl.registerFactory<CancelIssuanceUseCase>(MockCancelIssuanceUseCase.new);
    sl.registerFactory<CancelPidIssuanceUseCase>(MockCancelPidIssuanceUseCase.new);
    sl.registerFactory<CancelPinRecoveryUseCase>(MockCancelPinRecoveryUseCase.new);
    sl.registerFactory<CancelWalletTransferUseCase>(MockCancelWalletTransferUseCase.new);
    sl.registerFactory<ChangePinUseCase>(MockChangePinUseCase.new);
    sl.registerFactory<CheckIsPidUseCase>(MockCheckIsPidUseCase.new);
    sl.registerFactory<CheckIsValidPinUseCase>(MockCheckIsValidPinUseCase.new);
    sl.registerFactory<CheckNavigationPrerequisitesUseCase>(MockCheckNavigationPrerequisitesUseCase.new);
    sl.registerFactory<CheckPermissionUseCase>(MockCheckPermissionUseCase.new);
    sl.registerFactory<CheckPinUseCase>(MockCheckPinUseCase.new);
    sl.registerFactory<CompletePinRecoveryUseCase>(MockCompletePinRecoveryUseCase.new);
    sl.registerFactory<ConfirmWalletTransferUseCase>(MockConfirmWalletTransferUseCase.new);
    sl.registerFactory<ContinuePidIssuanceUseCase>(MockContinuePidIssuanceUseCase.new);
    sl.registerFactory<ContinuePinRecoveryUseCase>(MockContinuePinRecoveryUseCase.new);
    sl.registerFactory<CreatePinRecoveryRedirectUriUseCase>(MockCreatePinRecoveryRedirectUriUseCase.new);
    sl.registerFactory<CreateWalletUseCase>(MockCreateWalletUseCase.new);
    sl.registerFactory<DecodeQrUseCase>(MockDecodeQrUseCase.new);
    sl.registerFactory<DecodeUriUseCase>(MockDecodeUriUseCase.new);
    sl.registerFactory<DeleteWalletCardUseCase>(MockDeleteWalletCardUseCase.new);
    sl.registerFactory<DiscloseForIssuanceUseCase>(MockDiscloseForIssuanceUseCase.new);
    sl.registerFactory<FetchTourVideosUseCase>(MockFetchTourVideosUseCase.new);
    sl.registerFactory<GetAvailableBiometricsUseCase>(MockGetAvailableBiometricsUseCase.new);
    sl.registerFactory<GetMostRecentWalletEventUseCase>(MockGetMostRecentWalletEventUseCase.new);
    sl.registerFactory<GetPidCardsUseCase>(MockGetPidCardsUseCase.new);
    sl.registerFactory<GetPidIssuanceUrlUseCase>(MockGetPidIssuanceUrlUseCase.new);
    sl.registerFactory<GetPidRenewalUrlUseCase>(MockGetPidRenewalUrlUseCase.new);
    sl.registerFactory<GetRegistrationRevocationCodeUseCase>(MockGetRegistrationRevocationCodeUseCase.new);
    sl.registerFactory<GetRevocationCodeSavedUseCase>(MockGetRevocationCodeSavedUseCase.new);
    sl.registerFactory<GetRevocationCodeUseCase>(MockGetRevocationCodeUseCase.new);
    sl.registerFactory<GetSupportedBiometricsUseCase>(MockGetSupportedBiometricsUseCase.new);
    sl.registerFactory<GetVersionStringUseCase>(MockGetVersionStringUseCase.new);
    sl.registerFactory<GetWalletCardUseCase>(MockGetWalletCardUseCase.new);
    sl.registerFactory<GetWalletCardsUseCase>(MockGetWalletCardsUseCase.new);
    sl.registerFactory<GetWalletEventsForCardUseCase>(MockGetWalletEventsForCardUseCase.new);
    sl.registerFactory<GetWalletEventsUseCase>(MockGetWalletEventsUseCase.new);
    sl.registerFactory<GetWalletStateUseCase>(MockGetWalletStateUseCase.new);
    sl.registerFactory<InitWalletTransferUseCase>(MockInitWalletTransferUseCase.new);
    sl.registerFactory<IsBiometricLoginEnabledUseCase>(MockIsBiometricLoginEnabledUseCase.new);
    sl.registerFactory<IsWalletInitializedUseCase>(MockIsWalletInitializedUseCase.new);
    sl.registerFactory<IsWalletInitializedWithPidUseCase>(MockIsWalletInitializedWithPidUseCase.new);
    sl.registerFactory<IsWalletRegisteredAndUnlockedUseCase>(MockIsWalletRegisteredAndUnlockedUseCase.new);
    sl.registerFactory<LockWalletUseCase>(MockLockWalletUseCase.new);
    sl.registerFactory<MoveToReadyStateUseCase>(MockMoveToReadyStateUseCase.new);
    sl.registerFactory<ObserveCloseProximityConnectionUseCase>(MockObserveCloseProximityConnectionUseCase.new);
    sl.registerFactory<ObserveDashboardNotificationsUseCase>(MockObserveDashboardNotificationsUseCase.new);
    sl.registerFactory<ObserveMaintenanceStateUseCase>(MockObserveMaintenanceStateUseCase.new);
    sl.registerFactory<ObserveOsNotificationsUseCase>(MockObserveOsNotificationsUseCase.new);
    sl.registerFactory<ObservePushNotificationsSettingUseCase>(MockObservePushNotificationsSettingUseCase.new);
    sl.registerFactory<ObserveRecentWalletEventsUseCase>(MockObserveRecentWalletEventsUseCase.new);
    sl.registerFactory<ObserveShowTourBannerUseCase>(MockObserveShowTourBannerUseCase.new);
    sl.registerFactory<ObserveTransferSessionStateUseCase>(MockObserveTransferSessionStateUseCase.new);
    sl.registerFactory<ObserveVersionStateUsecase>(MockObserveVersionStateUsecase.new);
    sl.registerFactory<ObserveWalletCardDetailUseCase>(MockObserveWalletCardDetailUseCase.new);
    sl.registerFactory<ObserveWalletCardUseCase>(MockObserveWalletCardUseCase.new);
    sl.registerFactory<ObserveWalletCardsUseCase>(MockObserveWalletCardsUseCase.new);
    sl.registerFactory<ObserveWalletLockedUseCase>(MockObserveWalletLockedUseCase.new);
    sl.registerFactory<PairWalletTransferUseCase>(MockPairWalletTransferUseCase.new);
    sl.registerFactory<PerformPreNavigationActionsUseCase>(MockPerformPreNavigationActionsUseCase.new);
    sl.registerFactory<ReceiveWalletTransferUseCase>(MockReceiveWalletTransferUseCase.new);
    sl.registerFactory<RejectSignAgreementUseCase>(MockRejectSignAgreementUseCase.new);
    sl.registerFactory<RequestBiometricsUseCase>(MockRequestBiometricsUseCase.new);
    sl.registerFactory<RequestPermissionUseCase>(MockRequestPermissionUseCase.new);
    sl.registerFactory<ResetWalletUseCase>(MockResetWalletUseCase.new);
    sl.registerFactory<SetBiometricsUseCase>(MockSetBiometricsUseCase.new);
    sl.registerFactory<SetDirectOsNotificationCallbackUsecase>(MockSetDirectOsNotificationCallbackUsecase.new);
    sl.registerFactory<SetPushNotificationsSettingUseCase>(MockSetPushNotificationsSettingUseCase.new);
    sl.registerFactory<SetRevocationCodeSavedUseCase>(MockSetRevocationCodeSavedUseCase.new);
    sl.registerFactory<SetupMockedWalletUseCase>(MockSetupMockedWalletUseCase.new);
    sl.registerFactory<SkipWalletTransferUseCase>(MockSkipWalletTransferUseCase.new);
    sl.registerFactory<StartCloseProximityDisclosureUseCase>(MockStartCloseProximityDisclosureUseCase.new);
    sl.registerFactory<StartDisclosureUseCase>(MockStartDisclosureUseCase.new);
    sl.registerFactory<StartIssuanceUseCase>(MockStartIssuanceUseCase.new);
    sl.registerFactory<StartSignUseCase>(MockStartSignUseCase.new);
    sl.registerFactory<StartWalletTransferUseCase>(MockStartWalletTransferUseCase.new);
    sl.registerFactory<TourOverviewViewedUseCase>(MockTourOverviewViewedUseCase.new);
    sl.registerFactory<UnlockWalletWithPinUseCase>(MockUnlockWalletWithPinUseCase.new);

    isInitialized = true;
  }

  static MockConfigurationRepository _createConfigurationRepositoryMock() {
    final repository = MockConfigurationRepository();
    when(repository.observeAppConfiguration).thenAnswer(
      (_) => Stream.value(
        const FlutterAppConfiguration(
          idleLockTimeout: Duration(minutes: 2),
          idleWarningTimeout: Duration(minutes: 1),
          backgroundLockTimeout: Duration(minutes: 1),
          staticAssetsBaseUrl: 'https://example.com/',
          pidAttestationTypes: ['com.example.attestationType'],
          maintenanceWindow: null,
          version: '1',
          environment: 'test',
        ),
      ),
    );
    return repository;
  }

  static MockNetworkRepository _createNetworkRepositoryMock() {
    final mock = MockNetworkRepository();
    when(mock.hasInternet()).thenAnswer((realInvocation) async => true);
    CoreErrorExtension.networkRepository = mock;
    return mock;
  }

  static MockPidRepository _createPidRepositoryMock() {
    final mock = MockPidRepository();
    when(mock.getPidIssuanceUrl()).thenAnswer((_) async => kMockPidIssuanceUrl);
    return mock;
  }

  static MockTypedWalletCore _createTypedWalletCoreMock() {
    final mock = MockTypedWalletCore();
    when(mock.createPidIssuanceRedirectUri()).thenAnswer((_) async => kMockPidIssuanceUrl);
    return mock;
  }

  static T create<T extends Object>() {
    if (!isInitialized) initialize();
    return sl.get<T>();
  }
}
