import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../domain/usecase/card/get_pid_card_usecase.dart';
import '../domain/usecase/card/get_wallet_card_data_attributes_usecase.dart';
import '../domain/usecase/card/get_wallet_card_summary_usecase.dart';
import '../domain/usecase/card/get_wallet_card_timeline_attributes_usecase.dart';
import '../domain/usecase/card/get_wallet_card_usecase.dart';
import '../domain/usecase/card/get_wallet_cards_usecase.dart';
import '../domain/usecase/card/lock_wallet_usecase.dart';
import '../domain/usecase/card/log_card_interaction_usecase.dart';
import '../domain/usecase/card/observe_wallet_cards_usecase.dart';
import '../domain/usecase/card/wallet_add_issued_card_usecase.dart';
import '../domain/usecase/issuance/get_issuance_response_usecase.dart';
import '../domain/usecase/pin/check_is_valid_pin_usecase.dart';
import '../domain/usecase/pin/confirm_transaction_usecase.dart';
import '../domain/usecase/pin/get_available_pin_attempts_usecase.dart';
import '../domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import '../domain/usecase/qr/decode_qr_usecase.dart';
import '../domain/usecase/verification/get_verification_request_usecase.dart';
import '../domain/usecase/verification/get_verifier_policy_usecase.dart';
import '../domain/usecase/wallet/create_wallet_usecase.dart';
import '../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../domain/usecase/wallet/get_wallet_timeline_attributes_usecase.dart';

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
        RepositoryProvider<CheckIsAppInitializedUseCase>(
          create: (context) => CheckIsAppInitializedUseCase(context.read()),
        ),
        RepositoryProvider<UnlockWalletWithPinUseCase>(
          create: (context) => UnlockWalletWithPinUseCase(context.read()),
        ),
        RepositoryProvider<CreateWalletUseCase>(
          create: (context) => CreateWalletUseCase(context.read()),
        ),
        RepositoryProvider<CheckIsValidPinUseCase>(
          create: (context) => CheckIsValidPinUseCase(),
        ),
        RepositoryProvider<ConfirmTransactionUseCase>(
          create: (context) => ConfirmTransactionUseCase(context.read()),
        ),
        RepositoryProvider<GetAvailablePinAttemptsUseCase>(
          create: (context) => GetAvailablePinAttemptsUseCase(context.read()),
        ),
        RepositoryProvider<GetVerificationRequestUseCase>(
          create: (context) => GetVerificationRequestUseCase(context.read()),
        ),
        RepositoryProvider<GetRequestedAttributesFromWalletUseCase>(
          create: (context) => GetRequestedAttributesFromWalletUseCase(context.read()),
        ),
        RepositoryProvider<LogCardInteractionUseCase>(
          create: (context) => LogCardInteractionUseCase(context.read()),
        ),
        RepositoryProvider<GetVerifierPolicyUseCase>(
          create: (context) => GetVerifierPolicyUseCase(context.read()),
        ),
        RepositoryProvider<LockWalletUseCase>(
          create: (context) => LockWalletUseCase(context.read()),
        ),
        RepositoryProvider<GetWalletCardsUseCase>(
          create: (context) => GetWalletCardsUseCase(context.read()),
        ),
        RepositoryProvider<GetWalletCardUseCase>(
          create: (context) => GetWalletCardUseCase(context.read()),
        ),
        RepositoryProvider<ObserveWalletCardsUseCase>(
          create: (context) => ObserveWalletCardsUseCase(context.read()),
        ),
        RepositoryProvider<GetWalletCardSummaryUseCase>(
          create: (context) => GetWalletCardSummaryUseCase(
            context.read(),
            context.read(),
            context.read(),
          ),
        ),
        RepositoryProvider<GetWalletCardDataAttributesUseCase>(
          create: (context) => GetWalletCardDataAttributesUseCase(context.read()),
        ),
        RepositoryProvider<GetWalletCardTimelineAttributesUseCase>(
          create: (context) => GetWalletCardTimelineAttributesUseCase(context.read()),
        ),
        RepositoryProvider<DecodeQrUseCase>(
          create: (context) => DecodeQrUseCase(context.read()),
        ),
        RepositoryProvider<GetIssuanceResponseUseCase>(
          create: (context) => GetIssuanceResponseUseCase(context.read()),
        ),
        RepositoryProvider<WalletAddIssuedCardUseCase>(
          create: (context) => WalletAddIssuedCardUseCase(context.read(), context.read()),
        ),
        RepositoryProvider<GetPidCardUseCase>(
          create: (context) => GetPidCardUseCase(context.read()),
        ),
        RepositoryProvider<GetWalletTimelineAttributesUseCase>(
          create: (context) => GetWalletTimelineAttributesUseCase(context.read()),
        ),
      ],
      child: child,
    );
  }
}
