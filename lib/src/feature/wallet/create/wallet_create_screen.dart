import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../wallet_constants.dart';
import '../../../wallet_routes.dart';

/// Temporary screen to 'create' a new wallet.
/// TODO: Refine, Design, BLoC, UseCase; this is really just a placeholder for now.
class WalletCreateScreen extends StatelessWidget {
  const WalletCreateScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Setup Wallet')),
      body: Column(
        mainAxisSize: MainAxisSize.max,
        crossAxisAlignment: CrossAxisAlignment.center,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.wallet, size: 80),
          const SizedBox(height: 24),
          IntrinsicWidth(
            child: OutlinedButton(
              onPressed: () async {
                final navigator = Navigator.of(context);
                final success = await context.read<WalletRepository>().createWallet('123456');
                if (success) navigator.pushReplacementNamed(WalletRoutes.homeRoute);
              },
              child: const Text('Create Wallet'),
            ),
          ),
          const SizedBox(width: double.infinity, height: 16),
          const Text('Default pin: $kMockPin')
        ],
      ),
    );
  }
}
