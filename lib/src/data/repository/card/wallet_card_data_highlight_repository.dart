import '../../../domain/model/data_highlight.dart';

abstract class WalletCardDataHighlightRepository {
  Future<DataHighlight> getLatest(String cardId);
}
