import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../data/repository/card/mock_wallet_card_repository.dart';
import '../data/repository/card/wallet_card_repository.dart';
import '../data/repository/qr/mock_qr_repository.dart';
import '../data/repository/qr/qr_repository.dart';
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
          create: (context) => MockWalletCardRepository(),
        ),
        RepositoryProvider<VerificationRequestRepository>(
          create: (context) => MockVerificationRepository(),
        ),
        RepositoryProvider<QrRepository>(
          create: (context) => MockQrRepository(),
        )
      ],
      child: child,
    );
  }
}
