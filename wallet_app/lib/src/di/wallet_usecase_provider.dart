import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:internet_connection_checker/internet_connection_checker.dart';

import '../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../domain/usecase/app/impl/check_is_app_initialized_usecase_impl.dart';
import '../domain/usecase/card/get_pid_issuance_response_usecase.dart';
import '../domain/usecase/card/get_wallet_card_data_attributes_usecase.dart';
import '../domain/usecase/card/get_wallet_card_summary_usecase.dart';
import '../domain/usecase/card/get_wallet_card_timeline_attributes_usecase.dart';
import '../domain/usecase/card/get_wallet_card_update_issuance_request_id_usecase.dart';
import '../domain/usecase/card/get_wallet_card_usecase.dart';
import '../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../domain/usecase/card/impl/get_pid_issuance_response_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_card_data_attributes_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_card_summary_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_card_timeline_attributes_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_card_update_issuance_request_id_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_card_usecase_impl.dart';
import '../domain/usecase/card/impl/get_wallet_cards_usecase_impl.dart';
import '../domain/usecase/card/impl/lock_wallet_usecase_impl.dart';
import '../domain/usecase/card/impl/log_card_interaction_usecase_impl.dart';
import '../domain/usecase/card/impl/log_card_signing_usecase_impl.dart';
import '../domain/usecase/card/impl/observe_wallet_cards_usecase_impl.dart';
import '../domain/usecase/card/impl/wallet_add_issued_cards_usecase_impl.dart';
import '../domain/usecase/card/lock_wallet_usecase.dart';
import '../domain/usecase/card/log_card_interaction_usecase.dart';
import '../domain/usecase/card/log_card_signing_usecase.dart';
import '../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../domain/usecase/card/wallet_add_issued_cards_usecase.dart';
import '../domain/usecase/deeplink/decode_deeplink_usecase.dart';
import '../domain/usecase/deeplink/impl/decode_deeplink_usecase_impl.dart';
import '../domain/usecase/history/get_timeline_attribute_usecase.dart';
import '../domain/usecase/history/get_wallet_timeline_attributes_usecase.dart';
import '../domain/usecase/history/has_previously_interacted_with_organization_usecase.dart';
import '../domain/usecase/history/impl/get_timeline_attribute_usecase_impl.dart';
import '../domain/usecase/history/impl/get_wallet_timeline_attributes_usecase_impl.dart';
import '../domain/usecase/history/impl/has_previously_interacted_with_organization_usecase_impl.dart';
import '../domain/usecase/issuance/get_issuance_response_usecase.dart';
import '../domain/usecase/issuance/get_my_government_issuance_responses_usecase.dart';
import '../domain/usecase/issuance/impl/get_issuance_response_usecase_impl.dart';
import '../domain/usecase/issuance/impl/get_my_government_issuance_responses_usecase_impl.dart';
import '../domain/usecase/network/check_has_internet_usecase.dart';
import '../domain/usecase/network/impl/check_has_internet_usecase_impl.dart';
import '../domain/usecase/organization/get_organization_by_id_usecase.dart';
import '../domain/usecase/organization/impl/get_organization_by_id_usecase_impl.dart';
import '../domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import '../domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import '../domain/usecase/pid/impl/cancel_pid_issuance_usecase_impl.dart';
import '../domain/usecase/pid/impl/get_pid_issuance_url_usecase_impl.dart';
import '../domain/usecase/pid/impl/observe_pid_issuance_status_usecase_impl.dart';
import '../domain/usecase/pid/impl/update_pid_issuance_status_usecase_impl.dart';
import '../domain/usecase/pid/observe_pid_issuance_status_usecase.dart';
import '../domain/usecase/pid/update_pid_issuance_status_usecase.dart';
import '../domain/usecase/pin/check_is_valid_pin_usecase.dart';
import '../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../domain/usecase/pin/impl/check_is_valid_pin_usecase_impl.dart';
import '../domain/usecase/pin/impl/confirm_transaction_usecase_impl.dart';
import '../domain/usecase/pin/impl/unlock_wallet_with_pin_usecase_impl.dart';
import '../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../domain/usecase/qr/decode_qr_usecase.dart';
import '../domain/usecase/qr/impl/decode_qr_usecase_impl.dart';
import '../domain/usecase/sign/get_sign_request_usecase.dart';
import '../domain/usecase/sign/impl/get_sign_request_usecase_impl.dart';
import '../domain/usecase/verification/get_verification_request_usecase.dart';
import '../domain/usecase/verification/get_verifier_policy_usecase.dart';
import '../domain/usecase/verification/impl/get_verification_request_usecase_impl.dart';
import '../domain/usecase/verification/impl/get_verifier_policy_usecase_impl.dart';
import '../domain/usecase/wallet/create_wallet_usecase.dart';
import '../domain/usecase/wallet/get_first_names_usecase.dart';
import '../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../domain/usecase/wallet/get_requested_attributes_with_card_usecase.dart';
import '../domain/usecase/wallet/impl/create_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/impl/get_first_names_usecase_impl.dart';
import '../domain/usecase/wallet/impl/get_requested_attributes_from_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/impl/get_requested_attributes_with_card_usecase_impl.dart';
import '../domain/usecase/wallet/impl/is_wallet_initialized_with_pid_impl.dart';
import '../domain/usecase/wallet/impl/observe_wallet_lock_usecase_impl.dart';
import '../domain/usecase/wallet/impl/setup_mocked_wallet_usecase_impl.dart';
import '../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import '../domain/usecase/wallet/observe_wallet_lock_usecase.dart';
import '../domain/usecase/wallet/setup_mocked_wallet_usecase.dart';
import '../util/extension/bloc_extension.dart';

/// This widget is responsible for initializing and providing all `use cases`.
/// Most likely to be used once at the top (app) level, but notable below the
/// [WalletRepositoryProvider] as `use cases` will likely depend on one or more
/// `repositories`.
class WalletUseCaseProvider extends StatelessWidget {
  final Widget child;

  const WalletUseCaseProvider({required this.child, Key? key}) : super(key: key);

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
        RepositoryProvider<ConfirmTransactionUseCase>(
          create: (context) => ConfirmTransactionUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetVerificationRequestUseCase>(
          create: (context) => GetVerificationRequestUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetRequestedAttributesFromWalletUseCase>(
          create: (context) => GetRequestedAttributesFromWalletUseCaseImpl(context.read()),
        ),
        RepositoryProvider<LogCardInteractionUseCase>(
          create: (context) => LogCardInteractionUseCaseImpl(context.read()),
        ),
        RepositoryProvider<LogCardSigningUseCase>(
          create: (context) => LogCardSigningUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetVerifierPolicyUseCase>(
          create: (context) => GetVerifierPolicyUseCaseImpl(context.read()),
        ),
        RepositoryProvider<LockWalletUseCase>(
          create: (context) => LockWalletUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetFirstNamesUseCase>(
          create: (context) => GetFirstNamesUseCaseImpl(context.read()),
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
        RepositoryProvider<GetWalletCardSummaryUseCase>(
          create: (context) => GetWalletCardSummaryUseCaseImpl(
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<GetWalletCardDataAttributesUseCase>(
          create: (context) => GetWalletCardDataAttributesUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletCardTimelineAttributesUseCase>(
          create: (context) => GetWalletCardTimelineAttributesUseCaseImpl(context.read()),
        ),
        RepositoryProvider<DecodeQrUseCase>(
          create: (context) => DecodeQrUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetIssuanceResponseUseCase>(
          create: (context) => GetIssuanceResponseUseCaseImpl(context.read()),
        ),
        RepositoryProvider<WalletAddIssuedCardsUseCase>(
          create: (context) => WalletAddIssuedCardsUseCaseImpl(context.read(), context.read()),
        ),
        RepositoryProvider<GetPidIssuanceResponseUseCase>(
          create: (context) => GetPidIssuanceResponseUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CancelPidIssuanceUseCase>(
          create: (context) => CancelPidIssuanceUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetMyGovernmentIssuanceResponsesUseCase>(
          create: (context) => GetMyGovernmentIssuanceResponsesUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletTimelineAttributesUseCase>(
          create: (context) => GetWalletTimelineAttributesUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetTimelineAttributeUseCase>(
          create: (context) => GetTimelineAttributeUseCaseImpl(context.read()),
        ),
        RepositoryProvider<SetupMockedWalletUseCase>(
          create: (context) => SetupMockedWalletUseCaseImpl(
            context.read(),
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<GetSignRequestUseCase>(
          create: (context) => GetSignRequestUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetWalletCardUpdateIssuanceRequestIdUseCase>(
          create: (context) => GetWalletCardUpdateIssuanceRequestIdUseCaseImpl(context.read()),
        ),
        RepositoryProvider<DecodeDeeplinkUseCase>(
          create: (context) => DecodeDeeplinkUseCaseImpl(),
        ),
        RepositoryProvider<IsWalletInitializedWithPidUseCase>(
          create: (context) => IsWalletInitializedWithPidUseCaseImpl(context.read(), context.read()),
        ),
        RepositoryProvider<HasPreviouslyInteractedWithOrganizationUseCase>(
          create: (context) => HasPreviouslyInteractedWithOrganizationUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetOrganizationByIdUseCase>(
          create: (context) => GetOrganizationByIdUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetRequestedAttributesWithCardUseCase>(
          create: (context) => GetRequestedAttributesWithCardUseCaseImpl(context.read()),
        ),
        RepositoryProvider<GetPidIssuanceUrlUseCase>(
          create: (context) => GetPidIssuanceUrlUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObservePidIssuanceStatusUseCase>(
          create: (context) => ObservePidIssuanceStatusUseCaseImpl(context.read()),
        ),
        RepositoryProvider<UpdatePidIssuanceStatusUseCase>(
          create: (context) => UpdatePidIssuanceStatusUseCaseImpl(context.read()),
        ),
        RepositoryProvider<ObserveWalletLockUseCase>(
          create: (context) => ObserveWalletLockUseCaseImpl(context.read()),
        ),
        RepositoryProvider<CheckHasInternetUseCase>(
          lazy: false /* false to make sure [BlocExtensions.instance] is available */,
          create: (context) {
            final usecase = CheckHasInternetUseCaseImpl(Connectivity(), InternetConnectionChecker());
            BlocExtensions.checkHasInternetUseCase = usecase;
            return usecase;
          },
        ),
      ],
      child: child,
    );
  }
}
