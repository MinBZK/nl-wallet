import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/util/mapper/wallet/wallet_state_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  late WalletStateMapper mapper;

  setUp(() {
    mapper = WalletStateMapper();
  });

  group('map', () {
    test('maps core.WalletState_Ready to WalletStateReady', () {
      const input = core.WalletState_Ready();
      final output = mapper.map(input);
      expect(output, isA<WalletStateReady>());
    });

    test('maps core.WalletState_Locked to WalletStateLocked', () {
      final input = const core.WalletState_Locked(subState: core.WalletState_Ready());
      final output = mapper.map(input);
      expect(output, isA<WalletStateLocked>());
      expect((output as WalletStateLocked).substate, isA<WalletStateReady>());
    });

    test('maps core.WalletState_Transferring with role Source to WalletStateTransferring with role source', () {
      const input = core.WalletState_Transferring(role: core.WalletTransferRole.Source);
      final output = mapper.map(input);
      expect(output, isA<WalletStateTransferring>());
      expect((output as WalletStateTransferring).role, TransferRole.source);
    });

    test('maps core.WalletState_Transferring with role Destination to WalletStateTransferring with role target', () {
      const input = core.WalletState_Transferring(role: core.WalletTransferRole.Destination);
      final output = mapper.map(input);
      expect(output, isA<WalletStateTransferring>());
      expect((output as WalletStateTransferring).role, TransferRole.target);
    });

    test('maps core.WalletState_TransferPossible to WalletStateTransferPossible', () {
      const input = core.WalletState_TransferPossible();
      final output = mapper.map(input);
      expect(output, isA<WalletStateTransferPossible>());
    });

    test('maps core.WalletState_Registration to WalletStateRegistration', () {
      const input = core.WalletState_Registration();
      final output = mapper.map(input);
      expect(output, isA<WalletStateRegistration>());
    });

    test('maps core.WalletState_Disclosure to WalletStateDisclosure', () {
      const input = core.WalletState_Disclosure();
      final output = mapper.map(input);
      expect(output, isA<WalletStateDisclosure>());
    });

    test('maps core.WalletState_Issuance to WalletStateIssuance', () {
      const input = core.WalletState_Issuance();
      final output = mapper.map(input);
      expect(output, isA<WalletStateIssuance>());
    });

    test('maps core.WalletState_PinChange to WalletStatePinChange', () {
      const input = core.WalletState_PinChange();
      final output = mapper.map(input);
      expect(output, isA<WalletStatePinChange>());
    });

    test('maps core.WalletState_PinRecovery to WalletStatePinRecovery', () {
      const input = core.WalletState_PinRecovery();
      final output = mapper.map(input);
      expect(output, isA<WalletStatePinRecovery>());
    });

    test(
      'maps core.WalletState_WalletBlocked with reason RequiresAppUpdate to WalletStateWalletBlocked with reason requiresAppUpdate',
      () {
        const input = core.WalletState_WalletBlocked(reason: core.WalletBlockedReason.RequiresAppUpdate);
        final output = mapper.map(input);
        expect(output, isA<WalletStateWalletBlocked>());
        expect((output as WalletStateWalletBlocked).reason, WalletBlockedReason.requiresAppUpdate);
      },
    );

    test(
      'maps core.WalletState_WalletBlocked with reason BlockedByWalletProvider to WalletStateWalletBlocked with reason blockedByWalletProvider',
      () {
        const input = core.WalletState_WalletBlocked(reason: core.WalletBlockedReason.BlockedByWalletProvider);
        final output = mapper.map(input);
        expect(output, isA<WalletStateWalletBlocked>());
        expect((output as WalletStateWalletBlocked).reason, WalletBlockedReason.blockedByWalletProvider);
      },
    );

    test('maps core.WalletState_Empty to WalletStateEmpty', () {
      const input = core.WalletState_Empty();
      final output = mapper.map(input);
      expect(output, isA<WalletStateEmpty>());
    });
  });
}
