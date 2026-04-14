import 'package:wallet_core/core.dart';

import '../../domain/model/card/wallet_card.dart';

abstract class WalletDataSource {
  Future<List<WalletCard>> readAll();

  Future<WalletCard?> read(String attestationId);

  Stream<List<WalletCard>> observeCards();

  Future<WalletInstructionResult> delete(String pin, String attestationId);
}
