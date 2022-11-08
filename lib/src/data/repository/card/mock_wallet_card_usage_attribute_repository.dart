import 'package:collection/collection.dart';

import '../../../domain/model/usage_attribute.dart';
import 'wallet_card_usage_attribute_repository.dart';

class MockWalletCardUsageAttributeRepository implements WalletCardUsageAttributeRepository {
  MockWalletCardUsageAttributeRepository();

  @override
  Future<List<UsageAttribute>> getAll(String cardId) async {
    switch (cardId) {
      case '1':
        return _kMockCardIdOneUsageAttributes;
      case '2':
        return _kMockCardIdTwoUsageAttributes;
      default:
        throw UnimplementedError();
    }
  }

  @override
  Future<UsageAttribute?> getFiltered(String cardId, UsageStatus status) async {
    switch (cardId) {
      case '1':
        return _getFiltered(_kMockCardIdOneUsageAttributes, status);
      case '2':
        return _getFiltered(_kMockCardIdTwoUsageAttributes, status);
      default:
        throw UnimplementedError();
    }
  }

  UsageAttribute? _getFiltered(List<UsageAttribute> attributes, UsageStatus status) {
    return attributes.firstWhereOrNull((element) => element.status == status);
  }
}

final List<UsageAttribute> _kMockCardIdOneUsageAttributes = [
  UsageAttribute(
    value: 'Amsterdam Airport Schiphol',
    status: UsageStatus.rejected,
    dateTime: DateTime.now().subtract(const Duration(hours: 4)),
  ),
  UsageAttribute(
    value: 'Nederlandse Spoorwegen',
    status: UsageStatus.success,
    dateTime: DateTime.now().subtract(const Duration(days: 2)),
  ),
];

final List<UsageAttribute> _kMockCardIdTwoUsageAttributes = [
  UsageAttribute(
    value: 'Staatsloterij',
    status: UsageStatus.rejected,
    dateTime: DateTime.now().subtract(const Duration(minutes: 2)),
  ),
  UsageAttribute(
    value: 'Nederlandse Politie',
    status: UsageStatus.rejected,
    dateTime: DateTime.now().subtract(const Duration(hours: 3)),
  ),
];
