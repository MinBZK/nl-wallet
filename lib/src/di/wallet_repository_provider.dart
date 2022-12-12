import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/card/data_attribute_repository.dart';
import '../data/repository/card/data_highlight_repository.dart';
import '../data/repository/card/impl/data_attribute_repository_impl.dart';
import '../data/repository/card/impl/timeline_attribute_repository_impl.dart';
import '../data/repository/card/impl/wallet_card_repository_impl.dart';
import '../data/repository/card/mock/mock_data_highlight_repository.dart';
import '../data/repository/card/timeline_attribute_repository.dart';
import '../data/repository/card/wallet_card_repository.dart';
import '../data/repository/issuance/issuance_response_repository.dart';
import '../data/repository/issuance/mock/mock_issuance_response_repository.dart';
import '../data/repository/qr/mock_qr_repository.dart';
import '../data/repository/qr/qr_repository.dart';
import '../data/repository/sign/mock_sign_request_repository.dart';
import '../data/repository/sign/sign_request_repository.dart';
import '../data/repository/verification/mock_verification_request_repository.dart';
import '../data/repository/verification/verification_request_repository.dart';
import '../data/repository/wallet/mock_wallet_repository.dart';
import '../data/repository/wallet/wallet_repository.dart';

/// This widget is responsible for initializing and providing all `repositories`.
/// Most likely to be used once at the top (app) level.
class WalletRepositoryProvider extends StatelessWidget {
  final Widget child;

  const WalletRepositoryProvider({required this.child, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<WalletRepository>(
          create: (context) => MockWalletRepository(),
        ),
        RepositoryProvider<WalletCardRepository>(
          create: (context) => WalletCardRepositoryImpl(context.read()),
        ),
        RepositoryProvider<DataAttributeRepository>(
          create: (context) => DataAttributeRepositoryImpl(context.read()),
        ),
        RepositoryProvider<DataHighlightRepository>(
          create: (context) => MockDataHighlightRepository(),
        ),
        RepositoryProvider<TimelineAttributeRepository>(
          create: (context) => TimelineAttributeRepositoryImpl(context.read()),
        ),
        RepositoryProvider<VerificationRequestRepository>(
          create: (context) => MockVerificationRequestRepository(context.read()),
        ),
        RepositoryProvider<QrRepository>(
          create: (context) => MockQrRepository(),
        ),
        RepositoryProvider<IssuanceResponseRepository>(
          create: (context) => MockIssuanceResponseRepository(context.read(), context.read()),
        ),
        RepositoryProvider<SignRequestRepository>(
          create: (context) => MockSignRequestRepository(context.read()),
        ),
      ],
      child: child,
    );
  }
}
