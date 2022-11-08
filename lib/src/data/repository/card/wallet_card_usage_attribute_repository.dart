import '../../../domain/model/usage_attribute.dart';

abstract class WalletCardUsageAttributeRepository {
  Future<List<UsageAttribute>> getAll(String cardId);

  Future<UsageAttribute?> getFiltered(String cardId, UsageStatus status);
}
