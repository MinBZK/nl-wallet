import '../../../domain/model/data_highlight.dart';

abstract class DataHighlightRepository {
  Future<DataHighlight> getLatest(String cardId);
}
