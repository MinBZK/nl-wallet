import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/wallet_core/error/core_error_mapper.dart';
import 'package:wallet/src/wallet_core/typed/impl/typed_wallet_core_impl.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet/src/wallet_core/wallet_core.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  final WalletCore core = Mocks.create();

  late TypedWalletCore typedWalletCore;

  setUp(() {
    typedWalletCore = TypedWalletCoreImpl(core, CoreErrorMapper());
  });

  group('Observe Cards', () {
    test('observeCards should return 1 single (hardcoded) [Card]', () {
      expect(typedWalletCore.observeCards(), emits(hasLength(1)));
    });
  });
}
