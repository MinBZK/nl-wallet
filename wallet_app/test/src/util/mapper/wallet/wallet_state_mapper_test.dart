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
      final input = const core.WalletState_Locked(
        subState: core.WalletState_Ready(),
      );
      final output = mapper.map(input);
      expect(output, isA<WalletStateLocked>());
      expect((output as WalletStateLocked).substate, isA<WalletStateReady>());
    });

    test(
      'maps core.WalletState_Transferring with role Source to WalletStateTransferring with role source',
      () {
        const input = core.WalletState_Transferring(
          role: core.TransferRole.Source,
        );
        final output = mapper.map(input);
        expect(output, isA<WalletStateTransferring>());
        expect((output as WalletStateTransferring).role, TransferRole.source);
      },
    );

    test(
      'maps core.WalletState_Transferring with role Destination to WalletStateTransferring with role target',
      () {
        const input = core.WalletState_Transferring(
          role: core.TransferRole.Destination,
        );
        final output = mapper.map(input);
        expect(output, isA<WalletStateTransferring>());
        expect((output as WalletStateTransferring).role, TransferRole.target);
      },
    );

    test(
      'maps core.WalletState_TransferPossible to WalletStateTransferPossible',
      () {
        const input = core.WalletState_TransferPossible();
        final output = mapper.map(input);
        expect(output, isA<WalletStateTransferPossible>());
      },
    );

    test('maps core.WalletState_Registration to WalletStateRegistration', () {
      const input = core.WalletState_Unregistered();
      final output = mapper.map(input);
      expect(output, isA<WalletStateUnregistered>());
    });

    test('maps core.WalletState_Disclosure to WalletStateDisclosure', () {
      const input = core.WalletState_InDisclosureFlow();
      final output = mapper.map(input);
      expect(output, isA<WalletStateInDisclosureFlow>());
    });

    test('maps core.WalletState_Issuance to WalletStateIssuance', () {
      const input = core.WalletState_InIssuanceFlow();
      final output = mapper.map(input);
      expect(output, isA<WalletStateInIssuanceFlow>());
    });

    test('maps core.WalletState_PinChange to WalletStatePinChange', () {
      const input = core.WalletState_InPinChangeFlow();
      final output = mapper.map(input);
      expect(output, isA<WalletStateInPinChangeFlow>());
    });

    test('maps core.WalletState_PinRecovery to WalletStatePinRecovery', () {
      const input = core.WalletState_InPinRecoveryFlow();
      final output = mapper.map(input);
      expect(output, isA<WalletStateInPinRecoveryFlow>());
    });

    test(
      'maps core.WalletState_WalletBlocked with reason RequiresAppUpdate to WalletStateWalletBlocked with reason requiresAppUpdate',
      () {
        const input = core.WalletState_Blocked(
          canRegisterNewAccount: true,
          reason: core.BlockedReason.RequiresAppUpdate,
        );
        final output = mapper.map(input);
        expect(output, isA<WalletStateBlocked>());
        output as WalletStateBlocked;
        expect(output.reason, BlockedReason.requiresAppUpdate);
        expect(output.canRegisterNewAccount, isTrue);
      },
    );

    test(
      'maps core.WalletState_WalletBlocked with reason BlockedByWalletProvider to WalletStateWalletBlocked with reason blockedByWalletProvider',
      () {
        const input = core.WalletState_Blocked(
          canRegisterNewAccount: false,
          reason: core.BlockedReason.BlockedByWalletProvider,
        );
        final output = mapper.map(input);
        expect(output, isA<WalletStateBlocked>());
        output as WalletStateBlocked;
        expect(output.reason, BlockedReason.blockedByWalletProvider);
        expect(output.canRegisterNewAccount, isFalse);
      },
    );

    test('maps core.WalletState_Empty to WalletStateEmpty', () {
      const input = core.WalletState_Empty();
      final output = mapper.map(input);
      expect(output, isA<WalletStateEmpty>());
    });
  });
}
