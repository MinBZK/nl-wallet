import 'package:bluetooth/bluetooth.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:provider/single_child_widget.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/data/repository/help/help_content_repository.dart';
import 'package:wallet/src/data/repository/language/language_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/data/service/auto_lock_service.dart';
import 'package:wallet/src/data/service/event/app_event_coordinator.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/help/help_category.dart';
import 'package:wallet/src/domain/model/help/help_subcategory.dart';
import 'package:wallet/src/domain/model/permission/permission_check_result.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/domain/model/tour/tour_video.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/request_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/set_biometrics_usecase.dart';
import 'package:wallet/src/domain/usecase/card/get_pid_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/card/get_wallet_card_usecase.dart';
import 'package:wallet/src/domain/usecase/card/get_wallet_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_detail_usecase.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_card_usecase.dart';
import 'package:wallet/src/domain/usecase/card/observe_wallet_cards_usecase.dart';
import 'package:wallet/src/domain/usecase/close_proximity/observe_close_proximity_connection_usecase.dart';
import 'package:wallet/src/domain/usecase/close_proximity/start_close_proximity_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/start_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/event/get_most_recent_wallet_event_usecase.dart';
import 'package:wallet/src/domain/usecase/event/get_wallet_events_for_card_usecase.dart';
import 'package:wallet/src/domain/usecase/event/get_wallet_events_usecase.dart';
import 'package:wallet/src/domain/usecase/event/observe_recent_wallet_events_usecase.dart';
import 'package:wallet/src/domain/usecase/help/get_help_categories_usecase.dart';
import 'package:wallet/src/domain/usecase/help/get_help_topic_blocks_usecase.dart';
import 'package:wallet/src/domain/usecase/help/impl/get_help_categories_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/help/impl/get_help_topic_blocks_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/issuance/cancel_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/start_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import 'package:wallet/src/domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/observe_dashboard_notifications_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/observe_push_notifications_setting_usecase.dart';
import 'package:wallet/src/domain/usecase/notification/set_push_notifications_setting_usecase.dart';
import 'package:wallet/src/domain/usecase/permission/check_permission_usecase.dart';
import 'package:wallet/src/domain/usecase/permission/request_permission_usecase.dart';
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
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/qr/decode_qr_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/get_registration_revocation_code_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/get_revocation_code_saved_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/set_revocation_code_saved_usecase.dart';
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
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/create_wallet_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/get_wallet_state_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_registered_and_unlocked_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/lock_wallet_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/move_to_ready_state_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/banner/cubit/banner_cubit.dart';
import 'package:wallet/src/feature/card/data/argument/card_data_screen_argument.dart';
import 'package:wallet/src/feature/card/delete/argument/delete_card_screen_argument.dart';
import 'package:wallet/src/feature/card/detail/argument/card_detail_screen_argument.dart';
import 'package:wallet/src/feature/dashboard/argument/dashboard_screen_argument.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';
import 'package:wallet/src/feature/forgot_pin/argument/forgot_pin_screen_argument.dart';
import 'package:wallet/src/feature/help/argument/help_topic_screen_argument.dart';
import 'package:wallet/src/feature/history/detail/argument/history_detail_screen_argument.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';
import 'package:wallet/src/feature/login/argument/login_detail_screen_argument.dart';
import 'package:wallet/src/feature/organization/detail/argument/organization_detail_screen_argument.dart';
import 'package:wallet/src/feature/pin_timeout/argument/pin_timeout_screen_argument.dart';
import 'package:wallet/src/feature/policy/policy_screen_arguments.dart';
import 'package:wallet/src/feature/sign/argument/sign_screen_argument.dart';
import 'package:wallet/src/feature/tour/video/argument/tour_video_screen_argument.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../wallet_app_test_widget.dart';
import 'mocks/wallet_mock_data.dart';
import 'mocks/wallet_mocks.dart';

class MockOrganizationPolicyMapper extends Mock implements ContextMapper<OrganizationPolicy, String> {
  @override
  String map(BuildContext? context, OrganizationPolicy? input) => 'Mocked Agreement Text';
}

void main() {
  group('WalletRoutes Navigation', () {
    final allRoutes = WalletRoutes.allRoutes;

    for (final routeName in allRoutes) {
      testWidgets('should pump $routeName without errors', (tester) async {
        final arguments = _getMockArgumentsForRoute(routeName);
        final settings = RouteSettings(name: routeName, arguments: arguments);

        final route = WalletRoutes.routeFactory(settings);

        await tester.pumpWidgetWithAppWrapper(
          Builder(builder: (route as MaterialPageRoute).builder),
          providers: _getRequiredProviders(),
        );

        // Basic verification that the route pumped something
        if (route.settings.name != WalletRoutes.splashRoute) {
          expect(find.byType(Scaffold, skipOffstage: false), findsAny);
        }
      });
    }
  });
}

Object? _getMockArgumentsForRoute(String routeName) {
  switch (routeName) {
    case WalletRoutes.cardDataRoute:
      return const CardDataScreenArgument(cardId: 'id', cardTitle: 'Title').toMap();
    case WalletRoutes.cardDeleteRoute:
      return const DeleteCardScreenArgument(attestationId: 'id', cardTitle: 'Title').toMap();
    case WalletRoutes.cardDetailRoute:
      return CardDetailScreenArgument.fromId('id', null).toJson();
    case WalletRoutes.cardHistoryRoute:
      return 'id';
    case WalletRoutes.dashboardRoute:
      return const DashboardScreenArgument(cards: []).toJson();
    case WalletRoutes.disclosureRoute:
      return const DisclosureScreenArgument(type: DisclosureConnectionType.remote('uri', isQrCode: false));
    case WalletRoutes.forgotPinRoute:
      return const ForgotPinScreenArgument(useCloseButton: true).toJson();
    case WalletRoutes.historyDetailRoute:
      return HistoryDetailScreenArgument(walletEvent: WalletMockData.issuanceEvent).toMap();
    case WalletRoutes.issuanceRoute:
      return const IssuanceScreenArgument(isQrCode: false, uri: 'uri').toJson();
    case WalletRoutes.loginDetailRoute:
      return LoginDetailScreenArgument(
        organization: WalletMockData.organization,
        policy: WalletMockData.policy,
        cardRequests: const [],
        sharedDataWithOrganizationBefore: false,
      );
    case WalletRoutes.organizationDetailRoute:
      return OrganizationDetailScreenArgument(
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      ).toMap();
    case WalletRoutes.pinTimeoutRoute:
      return PinTimeoutScreenArgument(expiryTime: DateTime.now().add(const Duration(minutes: 5))).toMap();
    case WalletRoutes.policyRoute:
      return PolicyScreenArguments(relyingParty: WalletMockData.organization, policy: WalletMockData.policy);
    case WalletRoutes.signRoute:
      return const SignScreenArgument(uri: 'uri').toJson();
    case WalletRoutes.tourVideoRoute:
      return const TourVideoScreenArgument(videoTitle: 'Title', videoUrl: 'url', subtitleUrl: 'subtitle').toMap();
    case WalletRoutes.helpCategoryRoute:
      return const HelpCategory(id: 'cat', icon: 'help_outline', title: 'Cat', subtitle: 'Sub', subcategories: []);
    case WalletRoutes.helpSubcategoryRoute:
      return const HelpSubcategory(id: 'sub', title: 'Sub', groups: []);
    case WalletRoutes.helpTopicRoute:
      return const HelpTopicScreenArgument(topicId: 'cid');
    default:
      return null;
  }
}

List<SingleChildWidget> _getRequiredProviders() {
  return [
    ..._getRepositoryProviders(),
    ..._getServiceProviders(),
    ..._getUseCaseProviders(),
    ..._getOtherProviders(),
  ];
}

List<SingleChildWidget> _getRepositoryProviders() {
  return [
    RepositoryProvider<ConfigurationRepository>(create: (c) => Mocks.create()),
    RepositoryProvider<HelpContentRepository>(create: (c) => Mocks.create()),
    RepositoryProvider<LanguageRepository>(create: _createLanguageRepositoryMock),
    RepositoryProvider<WalletCardRepository>(create: (c) => Mocks.create()),
    RepositoryProvider<WalletRepository>(create: _createWalletRepositoryMock),
  ];
}

List<SingleChildWidget> _getServiceProviders() {
  return [
    RepositoryProvider<AppEventCoordinator>(create: (c) => Mocks.create()),
    RepositoryProvider<AppLifecycleService>(create: (c) => Mocks.create()),
    RepositoryProvider<AutoLockService>(create: (c) => Mocks.create()),
    RepositoryProvider<BiometricUnlockManager>(create: (c) => Mocks.create()),
    RepositoryProvider<Bluetooth>(create: (c) => Mocks.create()),
    RepositoryProvider<NavigationService>(create: (c) => Mocks.create()),
  ];
}

List<SingleChildWidget> _getUseCaseProviders() {
  return [
    RepositoryProvider<CancelDisclosureUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CancelIssuanceUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CancelPidIssuanceUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CancelPinRecoveryUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CancelWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ChangePinUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CheckIsPidUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CheckIsValidPinUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CheckNavigationPrerequisitesUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CheckPermissionUseCase>(create: _createCheckPermissionUseCaseMock),
    RepositoryProvider<CheckPinUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CompletePinRecoveryUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ConfirmWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ContinuePidIssuanceUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ContinuePinRecoveryUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CreatePinRecoveryRedirectUriUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<CreateWalletUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<DecodeQrUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<FetchTourVideosUseCase>(create: _createFetchTourVideosUseCaseMock),
    RepositoryProvider<GetAvailableBiometricsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetHelpCategoriesUseCase>(create: (c) => GetHelpCategoriesUseCaseImpl(c.read())),
    RepositoryProvider<GetHelpTopicBlocksUseCase>(create: (c) => GetHelpTopicBlocksUseCaseImpl(c.read())),
    RepositoryProvider<GetMostRecentWalletEventUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetPidCardsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetPidIssuanceUrlUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetPidRenewalUrlUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetRegistrationRevocationCodeUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetRevocationCodeSavedUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetSupportedBiometricsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetVersionStringUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetWalletCardUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetWalletCardsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetWalletEventsForCardUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetWalletEventsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<GetWalletStateUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<InitWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<IsWalletInitializedUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<IsWalletInitializedWithPidUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<IsWalletRegisteredAndUnlockedUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<LockWalletUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<MoveToReadyStateUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveCloseProximityConnectionUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveDashboardNotificationsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObservePushNotificationsSettingUseCase>(
      create: _createObservePushNotificationSettingsUseCaseMock,
    ),
    RepositoryProvider<ObserveRecentWalletEventsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveShowTourBannerUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveTransferSessionStateUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveVersionStateUsecase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveWalletCardDetailUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveWalletCardUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveWalletCardsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ObserveWalletLockedUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<PairWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<PerformPreNavigationActionsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<ReceiveWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<RejectSignAgreementUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<RequestBiometricsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<RequestPermissionUseCase>(create: _createRequestPermissionUseCaseMock),
    RepositoryProvider<SetBiometricsUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<SetPushNotificationsSettingUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<SetRevocationCodeSavedUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<SkipWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<StartCloseProximityDisclosureUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<StartDisclosureUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<StartIssuanceUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<StartSignUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<StartWalletTransferUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<TourOverviewViewedUseCase>(create: (c) => Mocks.create()),
    RepositoryProvider<UnlockWalletWithPinUseCase>(create: (c) => Mocks.create()),
  ];
}

List<SingleChildWidget> _getOtherProviders() {
  return [
    RepositoryProvider<BannerCubit>(create: _createBannerCubitMock),
    RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (c) => MockOrganizationPolicyMapper()),
  ];
}

BannerCubit _createBannerCubitMock(BuildContext context) => BannerCubit(
  context.read<ObserveShowTourBannerUseCase>(),
  context.read<ObserveVersionStateUsecase>(),
  context.read<ObserveDashboardNotificationsUseCase>(),
);

CheckPermissionUseCase _createCheckPermissionUseCaseMock(BuildContext context) {
  final usecase = Mocks.create<CheckPermissionUseCase>() as MockCheckPermissionUseCase;
  when(
    usecase.invoke(any),
  ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
  return usecase;
}

FetchTourVideosUseCase _createFetchTourVideosUseCaseMock(BuildContext context) {
  final useCase = Mocks.create<FetchTourVideosUseCase>() as MockFetchTourVideosUseCase;
  when(useCase.invoke()).thenAnswer(
    (_) async => Result.success([
      TourVideo(
        title: 'title'.untranslated,
        bulletPoints: 'bulletPoints'.untranslated,
        videoThumb: WalletAssets.icon_alert_data.untranslated,
        videoUrl: 'videoUrl'.untranslated,
        subtitleUrl: 'subtitleUrl'.untranslated,
      ),
    ]),
  );
  return useCase;
}

LanguageRepository _createLanguageRepositoryMock(BuildContext context) {
  final repository = Mocks.create<LanguageRepository>() as MockLanguageRepository;
  when(repository.preferredLocale).thenAnswer((_) => Stream.value(const Locale('en')));
  return repository;
}

ObservePushNotificationsSettingUseCase _createObservePushNotificationSettingsUseCaseMock(BuildContext context) {
  final usecase = Mocks.create<ObservePushNotificationsSettingUseCase>() as MockObservePushNotificationsSettingUseCase;
  when(usecase.invoke()).thenAnswer((_) => Stream.value(true));
  return usecase;
}

RequestPermissionUseCase _createRequestPermissionUseCaseMock(BuildContext context) {
  final usecase = Mocks.create<RequestPermissionUseCase>() as MockRequestPermissionUseCase;
  when(
    usecase.invoke(any),
  ).thenAnswer((_) async => const PermissionCheckResult(isGranted: true, isPermanentlyDenied: false));
  return usecase;
}

WalletRepository _createWalletRepositoryMock(BuildContext context) {
  final mock = Mocks.create<WalletRepository>();
  when(mock.isLockedStream).thenAnswer((_) => Stream.value(false));
  return mock;
}
