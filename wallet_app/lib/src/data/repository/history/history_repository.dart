import 'package:wallet_core/core.dart';

abstract class HistoryRepository {
  Future<List<WalletEvent>> getHistory();

  Future<List<WalletEvent>> getHistoryForCard(String docType);
}
