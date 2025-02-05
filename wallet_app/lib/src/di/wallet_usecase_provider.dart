import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:internet_connection_checker/internet_connection_checker.dart';
import 'package:local_auth/local_auth.dart';

import '../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../domain/usecase/app/impl/check_is_app_initialized_usecase_impl.dart';
import '../domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import '../domain/usecase/biometrics/get_supported_biometrics_usecase.dart';
import '../domain/usecase/biometrics/impl/get_available_biometrics_usecase_impl.dart';
import '../domain/usecase/biometrics/impl/get_supported_biometrics_usecase_impl.dart';
import '../domain/usecase/biometrics/impl/is_biometric_login_enabled_usecase_impl.dart';
import '../domain/usecase/biometrics/impl/request_biometrics_usecase_impl.dart';
import '../domain/usecase/biometrics/impl/set_biometrics_usecase_impl.dart';
import '../domain/usecase/biometrics/impl/unlock_wallet_with_biometrics_usecase_impl.dart';
import '../domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import '../domain/usecase/biometrics/request_biometrics_usecase.dart';
import '../domain/usecase/biometrics/set_biometrics_usecase.dart';
import '../domain/usecase/biometrics/unlock_wallet_with_biometrics_usecase.dart';
import '../domain/usecase/card/get_wallet_card_usecase.dart';
import '../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../domain/usecase/card/impl/get_wallet_card_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_cards_usecase_impl.dart';
import '../domain/usecase/card/impl/lock_wallet_usecase_impl.dart';
import '../domain/usecase/card/impl/observe_wallet_card_detail_usecase_impl.dart';
import '../domain/usecase/card/impl/observe_wallet_card_usecase_impl.dart';
import '../domain/usecase/card/impl/observe_wallet_cards_usecase_impl.dart';
import '../domain/usecase/card/lock_wallet_usecase.dart';
import '../domain/usecase/card/observe_wallet_card_detail_usecase.dart';
import '../domain/usecase/card/observe_wallet_card_usecase.dart';
import '../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../domain/usecase/disclosure/accept_disclosure_usecase.dart';
import '../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../domain/usecase/disclosure/impl/accept_disclosure_usecase_impl.dart';
import '../domain/usecase/disclosure/impl/cancel_disclosure_usecase_impl.dart';
import '../domain/usecase/disclosure/impl/start_disclosure_usecase_impl.dart';
import '../domain/usecase/disclosure/start_disclosure_usecase.dart';
import '../domain/usecase/event/get_wallet_events_for_card_usecase.dart';
import '../domain/usecase/event/get_wallet_events_usecase.dart';
import '../domain/usecase/event/impl/get_wallet_events_for_card_usecase_impl.dart';
import '../domain/usecase/event/impl/get_wallet_events_usecase_impl.dart';
import '../domain/usecase/event/impl/observe_recent_wallet_events_usecase_impl.dart';
import '../domain/usecase/event/observe_recent_wallet_events_usecase.dart';
import '../domain/usecase/history/impl/observe_recent_history_usecase_impl.dart';
import '../domain/usecase/history/observe_recent_history_usecase.dart';
import '../domain/usecase/issuance/accept_issuance_usecase.dart';
import '../domain/usecase/issuance/cancel_issuance_usecase.dart';
import '../domain/usecase/issuance/continue_issuance_usecase.dart';
import '../domain/usecase/issuance/impl/accept_issuance_usecase_impl.dart';
import '../domain/usecase/issuance/impl/cancel_issuance_usecase_impl.dart';
import '../domain/usecase/issuance/impl/continue_issuance_usecase_impl.dart';
import '../domain/usecase/issuance/impl/start_issuance_usecase_impl.dart';
import '../domain/usecase/issuance/start_issuance_usecase.dart';
import '../domain/usecase/navigation/check_navigation_prerequisites_usecase.dart';
import '../domain/usecase/navigation/impl/check_navigation_prerequisites_usecase_impl.dart';
import '../domain/usecase/navigation/impl/perform_pre_navigation_actions_usecase_impl.dart';
import '../domain/usecase/navigation/perform_pre_navigation_actions_usecase.dart';
import '../domain/usecase/network/check_has_internet_usecase.dart';
import '../domain/usecase/network/impl/check_has_internet_usecase_impl.dart';
import '../domain/usecase/permission/check_has_permission_usecase.dart';
import '../domain/usecase/permission/impl/check_has_permission_usecase_impl.dart';
import '../domain/usecase/pid/accept_offered_pid_usecase.dart';
import '../domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import '../domain/usecase/pid/continue_pid_issuance_usecase.dart';
import '../domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import '../domain/usecase/pid/impl/accept_offered_pid_usecase_impl.dart';
import '../domain/usecase/pid/impl/cancel_pid_issuance_usecase_impl.dart';
import '../domain/usecase/pid/impl/continue_pid_issuance_usecase_impl.dart';
import '../domain/usecase/pid/impl/get_pid_issuance_url_usecase_impl.dart';
import '../domain/usecase/pin/change_pin_usecase.dart';
import '../domain/usecase/pin/check_is_valid_pin_usecase.dart';
import '../domain/usecase/pin/disclose_for_issuance_usecase.dart';
import '../domain/usecase/pin/impl/change_pin_usecase_impl.dart';
import '../domain/usecase/pin/impl/check_is_valid_pin_usecase_impl.dart';
import '../domain/usecase/pin/impl/check_pin_usecase_impl.dart';
import '../domain/usecase/pin/impl/disclose_for_issuance_usecase_impl.dart';
import '../domain/usecase/pin/impl/unlock_wallet_with_pin_usecase_impl.dart';
import '../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../domain/usecase/qr/decode_qr_usecase.dart';
import '../domain/usecase/qr/impl/decode_qr_usecase_impl.dart';
import '../domain/usecase/sign/accept_sign_agreement_usecase.dart';
import '../domain/usecase/sign/impl/accept_sign_agreement_usecase_impl.dart';
import '../domain/usecase/sign/impl/reject_sign_agreement_usecase_impl.dart';
import '../domain/usecase/sign/impl/start_sign_usecase_impl.dart';
import '../domain/usecase/sign/reject_sign_agreement_usecase.dart';
import '../domain/usecase/sign/start_sign_usecase.dart';
import '../domain/usecase/update/impl/observe_version_state_usecase_impl.dart';
import '../domain/usecase/update/observe_version_state_usecase.dart';
import '../domain/usecase/uri/decode_uri_usecase.dart';
import '../domain/usecase/uri/impl/decode_uri_usecase_impl.dart';
import '../domain/usecase/version/get_version_string_usecase.dart';
import '../domain/usecase/version/impl/get_version_string_usecase_impl.dart';
import '../domain/usecase/wallet/create_wallet_usecase.dart';
import '../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../domain/usecase/wallet/get_requested_attributes_with_card_usecase.dart';
import '../domain/usecase/wallet/impl/create_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/impl/get_requested_attributes_from_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/impl/get_requested_attributes_with_card_usecase_impl.dart';
import '../domain/usecase/wallet/impl/is_wallet_initialized_with_pid_impl.dart';
import '../domain/usecase/wallet/impl/observe_wallet_locked_usecase_impl.dart';
import '../domain/usecase/wallet/impl/reset_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/impl/setup_mocked_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import '../domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import '../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../util/extension/bloc_extension.dart';
import '../util/extension/build_context_extension.dart';

/// This widget is responsible for initializing and providing all `use cases`.
/// Most likely to be used once at the top (app) level, but notable below the
/// [WalletRepositoryProvider] as `use cases` will likely depend on one or more
/// `repositories`.
class WalletUseCaseProvider extends StatelessWidget {
  final Widget child;

  const WalletUseCaseProvider({required this.child, super.key});

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<IsWalletInitializedUseCase>(
          create: (context) => IsWalletInitializedUseCaseImpl(context.read()),
        ),
        RepositoryProvider<UnlockWalletWithPinUseCase>(
          create: (context) => UnlockWalletWithPinUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CreateWalletUseCase>(
          create: (context) => CreateWalletUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CheckIsValidPinUseCase>(
          create: (context) => CheckIsValidPinUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetRequestedAttributesFromWalletUseCase>(
          create: (context) => GetRequestedAttributesFromWalletUseCaseImpl(context.read()),
        ),
        RepositoryProvider<LockWalletUseCase>(
          create: (context) => LockWalletUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletCardsUseCase>(
          create: (context) => GetWalletCardsUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletCardUseCase>(
          create: (context) => GetWalletCardUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveWalletCardsUseCase>(
          create: (context) => ObserveWalletCardsUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveWalletCardUseCase>(
          create: (context) => ObserveWalletCardUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveWalletCardDetailUseCase>(
          create: (context) => ObserveWalletCardDetailUseCaseImpl(
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<DecodeQrUseCase>(
          create: (context) => DecodeQrUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CancelPidIssuanceUseCase>(
          create: (context) => CancelPidIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<SetupMockedWalletUseCase>(
          create: (context) => SetupMockedWalletUseCaseImpl(
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<StartSignUseCase>(
          create: (context) => StartSignUseCaseImpl(context.read()),
        ),
        RepositoryProvider<DecodeUriUseCase>(
          create: (context) => DecodeUriUseCaseImpl(context.read()),
        ),
        RepositoryProvider<IsWalletInitializedWithPidUseCase>(
          create: (context) => IsWalletInitializedWithPidUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetRequestedAttributesWithCardUseCase>(
          create: (context) => GetRequestedAttributesWithCardUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetPidIssuanceUrlUseCase>(
          create: (context) => GetPidIssuanceUrlUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ContinuePidIssuanceUseCase>(
          create: (context) => ContinuePidIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveWalletLockedUseCase>(
          create: (context) => ObserveWalletLockedUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CheckHasInternetUseCase>(
          lazy: false /* false to make sure [BlocExtensions.instance] is available */,
          create: (context) {
            final usecase = CheckHasInternetUseCaseImpl(Connectivity(), InternetConnectionChecker());
            BlocExtensions.checkHasInternetUseCase = usecase;
            return usecase;
          },
        ),
        RepositoryProvider<AcceptOfferedPidUseCase>(
          create: (context) => AcceptOfferedPidUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ResetWalletUseCase>(
          create: (context) => ResetWalletUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CheckNavigationPrerequisitesUseCase>(
          create: (context) => CheckNavigationPrerequisitesUseCaseImpl(context.read()),
        ),
        RepositoryProvider<PerformPreNavigationActionsUseCase>(
          create: (context) => PerformPreNavigationActionsUseCaseImpl(context.read()),
        ),
        RepositoryProvider<StartDisclosureUseCase>(
          create: (context) => StartDisclosureUseCaseImpl(context.read()),
        ),
        RepositoryProvider<AcceptDisclosureUseCase>(
          create: (context) => AcceptDisclosureUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CancelDisclosureUseCase>(
          create: (context) => CancelDisclosureUseCaseImpl(context.read()),
        ),
        RepositoryProvider<StartIssuanceUseCase>(
          create: (context) => StartIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ContinueIssuanceUseCase>(
          create: (context) => ContinueIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<DiscloseForIssuanceUseCase>(
          create: (context) => DiscloseForIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<AcceptIssuanceUseCase>(
          create: (context) => AcceptIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CancelIssuanceUseCase>(
          create: (context) => CancelIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<AcceptSignAgreementUseCase>(
          create: (context) => AcceptSignAgreementUseCaseImpl(context.read()),
        ),
        RepositoryProvider<RejectSignAgreementUseCase>(
          create: (context) => RejectSignAgreementUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveRecentHistoryUseCase>(
          create: (context) => ObserveRecentHistoryUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveRecentWalletEventsUseCase>(
          create: (context) => ObserveRecentWalletEventsUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletEventsUseCase>(
          create: (context) => GetWalletEventsUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletEventsForCardUseCase>(
          create: (context) => GetWalletEventsForCardUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CheckHasPermissionUseCase>(
          create: (context) => CheckHasPermissionUseCaseImpl(),
        ),
        RepositoryProvider<GetAvailableBiometricsUseCase>(
          create: (context) => GetAvailableBiometricsUseCaseImpl(
            LocalAuthentication(),
            context.theme.platform,
          ),
        ),
        RepositoryProvider<SetBiometricsUseCase>(
          create: (context) => SetBiometricsUseCaseImpl(
            LocalAuthentication(),
            context.theme.platform,
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<GetSupportedBiometricsUseCase>(
          create: (context) => GetSupportedBiometricsUseCaseImpl(LocalAuthentication()),
        ),
        RepositoryProvider<CheckPinUseCase>(
          create: (context) => CheckPinUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ChangePinUseCase>(
          create: (context) => ChangePinUseCaseImpl(context.read()),
        ),
        RepositoryProvider<IsBiometricLoginEnabledUseCase>(
          create: (context) => IsBiometricLoginEnabledUseCaseImpl(
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<RequestBiometricsUsecase>(
          create: (context) => RequestBiometricsUsecaseImpl(
            LocalAuthentication(),
            context.read(),
            context.theme.platform,
          ),
        ),
        RepositoryProvider<UnlockWalletWithBiometricsUseCase>(
          create: (context) => UnlockWalletWithBiometricsUseCaseImpl(
            context.read(),
            context.read(),
            LocalAuthentication(),
            context.theme.platform,
            context.read(),
          ),
        ),
        RepositoryProvider<ObserveVersionStateUsecase>(
          create: (context) => ObserveVersionStateUsecaseImpl(
            context.read(),
          ),
        ),
        RepositoryProvider<GetVersionStringUseCase>(
          create: (context) => GetVersionStringUseCaseImpl(context.read()),
        ),
      ],
      child: child,
    );
  }
}
