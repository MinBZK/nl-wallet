import '../../../domain/model/usage_attribute.dart';

abstract class WalletCardUsageAttributeRepository {
  Future<UsageAttribute> getAll(String cardId);

  Future<UsageAttribute> getLatest(String cardId);
}
