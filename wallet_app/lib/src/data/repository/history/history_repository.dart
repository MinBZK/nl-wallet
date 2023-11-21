import '../../../../bridge_generated.dart';

abstract class HistoryRepository {
  Future<List<WalletEvent>> getHistory();

  Future<List<WalletEvent>> getHistoryForCard(String docType);
}
